
use std::{
	collections::{HashMap, HashSet}, 
	sync::{
		Arc, 
		Weak, 
		mpsc::{Receiver, TryRecvError},
	},
};

use sdl3::{
	VideoSubsystem, 
	video::WindowBuilder,
};
use uuid::Uuid;
use vulkano::{
	Version, VulkanLibrary, 
	command_buffer::allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo}, 
	device::{
		Device, 
		DeviceCreateInfo, 
		DeviceExtensions, 
		DeviceFeatures, 
		Queue, 
		QueueCreateInfo, 
		QueueFlags, 
		physical::{PhysicalDevice, PhysicalDeviceType},
	}, instance::{Instance, InstanceCreateInfo, InstanceExtensions}, 
	memory::allocator::StandardMemoryAllocator, 
	render_pass::RenderPass, 
	swapchain::Surface as VSurface, 
	sync::{
		GpuFuture, 
		future::FenceSignalFuture, 
	},
};

use crate::{
	engine_command::{EngineCommand, RenderInstruction}, 
	render_engine::RenderEngineCreateInfo, 
	vulkan::{
		mesh_data::MeshData, 
		surface::Surface,
	},
};

pub struct RenderThread {
	command_channel: 		Receiver<EngineCommand>,
	should_close:			bool,

	pub mesh_data:			HashMap<Uuid, MeshData>,
	#[expect(unused)]
	pub shader_modules:		HashMap<Uuid, ()>,
	#[expect(unused)]
	pub pipelines:			HashMap<Uuid, ()>,

	#[expect(unused)]
	pub render_passes: 		Vec<(Weak<RenderPass>, Uuid)>,
	#[expect(unused)]
	pub linked_pipelines:	HashMap<Uuid, HashMap<Uuid, ()>>,

	pub surfaces:			HashMap<Uuid, Box<dyn Surface>>,

	pub video:				VideoSubsystem,

	pub instance:			Arc<Instance>,
	pub device:				Arc<Device>,
	pub graphics_queue: 	Arc<Queue>,
	pub graphics_operation: Operation,
	pub transfer_queue: 	Arc<Queue>,
	pub transfer_operation: Operation,

	pub buffer_allocator:	Arc<StandardMemoryAllocator>,
	pub command_allocator:	Arc<StandardCommandBufferAllocator>,
}

#[derive(Clone)]
pub struct Operation {
	pub operation_type: OperationType,
	pub future: Option<Arc<FenceSignalFuture<Box<dyn GpuFuture + Send>>>>,
}

#[derive(PartialEq)]
#[derive(Clone, Copy)]
pub enum OperationType {
	Graphics,
	Transfer,
}

impl RenderThread {
	pub fn new(create_info: RenderEngineCreateInfo, command_channel: Receiver<EngineCommand>) -> Result<Self, ()> {
		let sdl = sdl3::init().map_err(|_| ())?;
		let video = sdl.video().map_err(|_| ())?;

		let library = VulkanLibrary::new().map_err(|_| ())?;

		let instance = Instance::new(
			library, 
			InstanceCreateInfo {
				application_name: Some(create_info.app_name.clone()),
				application_version: RenderEngineCreateInfo::to_version(create_info.app_version),
				engine_name: Some(env!("CARGO_PKG_NAME").to_string()),
				engine_version: RenderEngineCreateInfo::engine_version(),
				enabled_extensions: create_info.generate_instance_extensions(&video)?,
				..Default::default()
			}
		).map_err(|_| ())?;

		let device_extensions = create_info.generate_device_extensions();
		let device_features = create_info.generate_device_features();

		let physical_device = Self::select_physical_device(instance.clone(), device_extensions, device_features)?;

		let (device, (graphics_queue, transfer_queue)) = Self::select_queues(physical_device, device_extensions, device_features)?;

		let buffer_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
		let command_allocator = Arc::new(StandardCommandBufferAllocator::new(device.clone(), StandardCommandBufferAllocatorCreateInfo::default()));

		Ok(RenderThread {
			command_channel: 	command_channel,
			should_close:		false,

			mesh_data:			HashMap::new(),
			shader_modules:		HashMap::new(),
			pipelines:			HashMap::new(),
			
			render_passes:		Vec::new(),
			linked_pipelines:	HashMap::new(),

			surfaces:			HashMap::new(),

			video:				video,

			instance:			instance,
			device:				device,
			graphics_queue: 	graphics_queue,
			graphics_operation:	Operation { future: None, operation_type: OperationType::Graphics },
			transfer_queue: 	transfer_queue,
			transfer_operation:	Operation { future: None, operation_type: OperationType::Transfer },

			buffer_allocator:	buffer_allocator,
			command_allocator:	command_allocator,
		})
	}

	pub fn should_close(&self) -> bool { self.should_close }

	pub fn process_commands(&mut self) -> () {
		loop {
			let result = self.command_channel.try_recv();
			if let Err(TryRecvError::Empty) = result { return; }
			let command = result.unwrap(); 

			match command {
				EngineCommand::ProcessRenderInstructionBuffer { instructions, response } => response.send(self.process_render_instruction_buffer(instructions)),
				EngineCommand::MeshDataCommand(command) => self.process_mesh_data_command(command),
				EngineCommand::ShaderCommand(_command) => todo!(),
				EngineCommand::WindowSurfaceCommand(command) => self.process_window_surface_command(command),
				EngineCommand::DropRenderThread => { self.should_close = true; }
			}
		}
	}

	fn process_render_instruction_buffer(&mut self, instructions: Box<[RenderInstruction]>) -> Result<(), ()> {
		let mut active_surface = None;

		for instruction in instructions {
			match instruction {
				RenderInstruction::BeginRendering { uuid } => {
					active_surface = self.surfaces.get_mut(&uuid);
					active_surface.as_mut().unwrap().begin_rendering(&self.command_allocator, &self.graphics_queue)?;
				},
				RenderInstruction::EndRendering => {
					self.graphics_operation = active_surface.as_mut().unwrap().end_rendering(&self.graphics_queue, self.graphics_operation.clone())?;
				},
			}
		}

		Ok(())
	}

	fn select_physical_device(instance: Arc<Instance>, device_extensions: DeviceExtensions, device_features: DeviceFeatures) -> Result<Arc<PhysicalDevice>, ()> {
		Ok(instance.enumerate_physical_devices().map_err(|_| ())?
			.filter(|pd| pd.supported_extensions().intersection(&device_extensions) == device_extensions)
			.filter(|pd| pd.supported_features().intersection(&device_features) == device_features)
			.map(|pd| {
				let mut flags = QueueFlags::empty();
				pd.queue_family_properties().iter()
					.for_each(|qfp| flags = flags.union(qfp.queue_flags));
				(pd, flags)
			})
			.filter(|(_, qf)| qf.contains(QueueFlags::GRAPHICS))
			.filter(|(_, qf)| qf.contains(QueueFlags::TRANSFER))
			.min_by_key(|(pd, _)| {
				match pd.properties().device_type {
					PhysicalDeviceType::DiscreteGpu => 0,
					PhysicalDeviceType::IntegratedGpu => 1,
					PhysicalDeviceType::VirtualGpu => 2,
					PhysicalDeviceType::Cpu => 3,
					PhysicalDeviceType::Other => 4,
					_ => 5,
				}
			})
			.map(|(pd, _)| pd)
			.ok_or(())?)
	}

	fn select_queues(physical_device: Arc<PhysicalDevice>, device_extensions: DeviceExtensions, device_features: DeviceFeatures) -> Result<(Arc<Device>, (Arc<Queue>, Arc<Queue>)), ()> {
		let mut queue_set = HashSet::new();

		let (graphics_queue_family, graphics_queue_index) = physical_device.queue_family_properties().iter()
			.enumerate()
			.filter(|(_, qfp)| qfp.queue_flags.contains(QueueFlags::GRAPHICS))
			.max_by_key(|(_, qfp)| qfp.queue_count)
			.map(|(i, _)| (i as u32, 0u32))
			.ok_or(())?;
		queue_set.insert((graphics_queue_family, graphics_queue_index));

		let (transfer_queue_family, transfer_queue_index) = physical_device.queue_family_properties().iter()
			.enumerate()
			.filter(|(i, _)| *i as u32 != graphics_queue_family)
			.filter(|(_, qfp)| qfp.queue_flags.contains(QueueFlags::TRANSFER))
			.min_by_key(|(_, qfp)| qfp.queue_flags.count())
			.map(|(i, _)| (i as u32, 0u32))
			.unwrap_or(if physical_device.queue_family_properties()[graphics_queue_family as usize].queue_count != 1 { (graphics_queue_family, 1u32) } else { (graphics_queue_family, 0u32) });
		queue_set.insert((transfer_queue_family, transfer_queue_index));

		let mut queue_create_infos = Vec::new();
		queue_set.iter().for_each(|(family, _)| queue_create_infos.push(QueueCreateInfo { queue_family_index: *family, ..Default::default() }) );

		let (device, queues) = Device::new(
			physical_device, 
			DeviceCreateInfo {
				queue_create_infos: queue_create_infos,
				enabled_extensions: device_extensions,
				enabled_features: device_features,
				..Default::default()
			}
		).map_err(|_| ())?;
		
		let mut queue_set = HashMap::new();
		for queue in queues {
			queue_set.entry(queue.queue_family_index())
				.or_insert(Vec::new())
				.push(queue);
		}

		let graphics_queue = queue_set[&graphics_queue_family][graphics_queue_index as usize].clone();
		let transfer_queue = queue_set[&transfer_queue_family][transfer_queue_index as usize].clone();

		return Ok((device, (graphics_queue, transfer_queue)));
	}
}

impl Operation {
	pub fn graphics(future: Arc<FenceSignalFuture<Box<dyn GpuFuture + Send>>>) -> Self {
		Operation {
			operation_type: OperationType::Graphics,
			future: 		Some(future),
		}
	}

	pub fn transfer(future: Arc<FenceSignalFuture<Box<dyn GpuFuture + Send>>>) -> Self {
		Operation {
			operation_type: OperationType::Transfer,
			future:			Some(future),
		}
	}

	pub fn needs_semaphore(&self, operation_type: OperationType) -> bool {
		if self.future.is_none() { return false }
		operation_type != self.operation_type
	}

	pub fn wait(&self) -> () {
		if self.future.is_some() {
			let _ = self.future.as_ref().unwrap().wait(None);
		}
	}

	pub fn cleanup_finished(&mut self) -> () {
		if self.future.is_none() { return }
		self.future.as_mut().unwrap().cleanup_finished();
	}
}

impl RenderEngineCreateInfo {
	fn to_version(version: [u32; 3]) -> Version {
		Version {
			major: version[0],
			minor: version[1],
			patch: version[2],
		}
	}

	fn engine_version() -> Version {
		Self::to_version([
			env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap(),
			env!("CARGO_PKG_VERSION_MINOR").parse().unwrap(),
			env!("CARGO_PKG_VERSION_PATCH").parse().unwrap(),
		])
	}

	fn generate_instance_extensions(&self, video: &VideoSubsystem) -> Result<InstanceExtensions, ()> {
		let window = WindowBuilder::new(video, "", 1, 1)
			.hidden()
			.build()
			.map_err(|_| ())?;
		let required_surface_extensions = VSurface::required_extensions(&window).map_err(|_| ())?;

		let instance_extensions = InstanceExtensions {
			khr_surface: true,
			..Default::default()
		};

		Ok(required_surface_extensions.union(&instance_extensions))
	}

	fn generate_device_extensions(&self) -> DeviceExtensions {
		DeviceExtensions {
			khr_dynamic_rendering: 	true,
			khr_swapchain: 			true,
			..Default::default()
		}
	}

	fn generate_device_features(&self) -> DeviceFeatures {
		DeviceFeatures {
			..Default::default()
		}
	}
}

macro_rules! start_render_thread {
	($create_info: expr, $command_channel: expr, $response: expr) => {
		{
			use crate::vulkan::render_thread::RenderThread;

			let result = RenderThread::new($create_info, $command_channel);
			let response = if let Err(e) = result { Err(e) } else { Ok(()) };
			$response.send(response);
			let mut render_thread = result.unwrap();

			while !render_thread.should_close() {
				render_thread.process_commands();
			}
		}
	};
}
pub(crate) use start_render_thread as start_vulkan_render_thread;