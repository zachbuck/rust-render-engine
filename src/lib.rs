
use std::{
	collections::HashMap, 
	sync::Arc
};

use shaderc::Compiler;
use uuid::Uuid;
use vulkano::{
	Version, 
	VulkanLibrary, 
	command_buffer::allocator::StandardCommandBufferAllocator, 
	device::{Device, DeviceCreateInfo, DeviceExtensions, DeviceFeatures, Queue, QueueCreateInfo, QueueFlags}, 
	instance::{Instance, InstanceCreateInfo, InstanceExtensions}, 
	memory::allocator::StandardMemoryAllocator, 
	sync::{self, GpuFuture}
};

use crate::{
	mesh_data::MeshDataInternal, 
	render_surface::RenderSurface, 
	render_target::RenderTarget, 
	shader::{GraphicsProgramInternal, ShaderInternal}
};

/*	TODO
	- Add error types to Results
	- Make it so get_image_surface_data isn't blocking?
 */

pub mod mesh_data;
pub mod render_surface;
pub mod render_target;
pub mod shader;

pub struct RenderEngine {
	mesh_data: HashMap<Uuid, MeshDataInternal>,
	shaders: HashMap<Uuid, ShaderInternal>,
	graphics_programs: HashMap<Uuid, GraphicsProgramInternal>,

	render_surfaces: HashMap<Uuid, RenderSurface>,
	render_targets: HashMap<Uuid, RenderTarget>,

	compiler: Compiler,

	device: Arc<Device>,
	graphics_queue: Arc<Queue>,
	transfer_queue: Arc<Queue>,
	graphics_operation: Option<Box<dyn GpuFuture>>,
	transfer_operation: Option<Box<dyn GpuFuture>>,

	command_allocator: Arc<StandardCommandBufferAllocator>,
	buffer_allocator: Arc<StandardMemoryAllocator>,
}

impl RenderEngine {
	pub fn new(mut create_info: RenderEngineCreateInfo) -> Result<RenderEngine, ()> {
		let library = VulkanLibrary::new().unwrap();
		
		let instance = Instance::new(
			library,
			InstanceCreateInfo {
				enabled_extensions: create_info.instance_extensions,

				application_name: create_info.app_name.clone(),
				application_version: create_info.get_app_version(),

				engine_name: RenderEngineCreateInfo::get_engine_name(),
				engine_version: RenderEngineCreateInfo::get_engine_version(),

				..Default::default()
			}
		).unwrap();

		let physical_device = instance.enumerate_physical_devices().unwrap()
			// filter for devices that support dynamic rendering
			.filter(|pd| pd.api_version() >= Version::V1_3 || pd.supported_extensions().khr_dynamic_rendering)
			// filter for devices that support requested device extensions
			.filter(|pd| pd.supported_extensions().contains(&create_info.device_extensions))
			// filter for devices that support graphics operations
			.filter(|pd| pd.queue_family_properties().iter().find(|qf| qf.queue_flags.intersects(QueueFlags::GRAPHICS)).is_some())
			// prioritize physical devices that are typically going to be stronger/faster
			.min_by_key(|pd| {
				match pd.properties().device_type {
						vulkano::device::physical::PhysicalDeviceType::IntegratedGpu => 0,
						vulkano::device::physical::PhysicalDeviceType::DiscreteGpu => 1,
						vulkano::device::physical::PhysicalDeviceType::VirtualGpu => 2,
						vulkano::device::physical::PhysicalDeviceType::Cpu => 3,
						vulkano::device::physical::PhysicalDeviceType::Other => 4,
						_ => 5,
					}
			}).unwrap();

		if physical_device.api_version() < Version::V1_3 {
			create_info.device_extensions.khr_dynamic_rendering = true;
		}

		let graphics_queue_family_index = physical_device.queue_family_properties().iter().enumerate()
			// filter by queue families which support graphics
			.filter(|(_, qf)| qf.queue_flags.intersects(QueueFlags::GRAPHICS))
			// select the one with the most available queues
			.max_by_key(|(_, qf)| qf.queue_count)
			// get the index of the queue family
			.map(|(i, _)| i as u32).unwrap();

		let transfer_queue_family_index = physical_device.queue_family_properties().iter().enumerate()
			// filter out the graphics queue index
			.filter(|(i, _)| *i as u32 != graphics_queue_family_index)
			// filter by queue families which support transfer operations
			.filter(|(_, qf)| qf.queue_flags.intersects(QueueFlags::TRANSFER))
			// select the one with the most available queues
			.max_by_key(|(_, qf)| qf.queue_count)
			// get the index of the queue family
			.map(|(i, _)| i as u32)
			// if no such queue exists, use the graphics queue family
			.unwrap_or(graphics_queue_family_index);

		let mut queue_create_infos = Vec::new();
		queue_create_infos.push(QueueCreateInfo {
			queue_family_index: graphics_queue_family_index,
			..Default::default()
		});
		if transfer_queue_family_index != graphics_queue_family_index {
			queue_create_infos.push(QueueCreateInfo {
				queue_family_index: transfer_queue_family_index,
				..Default::default()
			})
		}

		let (device, mut queues) = Device::new(
			physical_device,
			DeviceCreateInfo {
				queue_create_infos: queue_create_infos,
				enabled_extensions: create_info.device_extensions,
				enabled_features: create_info.device_features,
				..Default::default()
			}
		).unwrap();

		let graphics_queue = queues.next().unwrap();
		let transfer_queue = queues.next().unwrap_or(graphics_queue.clone());

		let command_allocator = Arc::new(StandardCommandBufferAllocator::new(device.clone(), Default::default()));
		let buffer_allocator = Arc::new(StandardMemoryAllocator::new(device.clone(), Default::default()));

		let compiler = Compiler::new().unwrap();

		return Ok(RenderEngine {
			mesh_data: HashMap::new(),
			shaders: HashMap::new(),
			graphics_programs: HashMap::new(),
			render_surfaces: HashMap::new(),
			render_targets: HashMap::new(),
			compiler: compiler,
			device: device.clone(),
			graphics_queue: graphics_queue,
			transfer_queue: transfer_queue,
			graphics_operation: Some(sync::now(device.clone()).boxed()),
			transfer_operation: Some(sync::now(device).boxed()),
			command_allocator: command_allocator,
			buffer_allocator: buffer_allocator,
		});
	}
}

pub struct RenderEngineCreateInfo {
	app_name: Option<String>,
	app_version: Option<[u32; 3]>,

	instance_extensions: InstanceExtensions,
	device_extensions: DeviceExtensions,
	device_features: DeviceFeatures
}

impl RenderEngineCreateInfo {
	pub fn with_app_name(&mut self, name: String) { self.app_name = Some(name); }
	pub fn with_app_version(&mut self, major: u32, minor: u32, patch: u32) { self.app_version = Some([major, minor, patch]); }

	fn get_app_version(&self) -> Version { 
		let version = self.app_version.unwrap_or([0, 0, 0]);

		return Version {
			major: version[0],
			minor: version[1],
			patch: version[2]
		}
	}
	fn get_engine_name() -> Option<String> { Some(env!("CARGO_PKG_NAME").to_string()) }
	fn get_engine_version() -> Version {
		const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
		let version_numbers = APP_VERSION.split(".")
			.map(|s| str::parse::<u32>(s).unwrap())
			.collect::<Vec<_>>();

		return Version {
			major: version_numbers[0],
			minor: version_numbers[1],
			patch: version_numbers[2]
		};
	}
}

impl Default for RenderEngineCreateInfo {
	fn default() -> Self {
		Self { 
			app_name: None,
			app_version: None,
			instance_extensions: InstanceExtensions::empty(),
			device_extensions: DeviceExtensions::empty(),
			device_features: DeviceFeatures {
				dynamic_rendering: true,
				..Default::default()
			}
		}
	}
}