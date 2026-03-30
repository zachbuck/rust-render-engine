
use std::{
    collections::HashMap, 
    sync::{
        Arc, 
        mpsc::Receiver
    }
};

use uuid::Uuid;
use vulkano::{
    VulkanLibrary, 
    command_buffer::allocator::StandardCommandBufferAllocator, 
    descriptor_set::allocator::StandardDescriptorSetAllocator, 
    device::{
        Device, 
        DeviceCreateInfo, 
        Queue, 
        QueueCreateInfo, 
        QueueFlags, 
        physical::PhysicalDeviceType
    }, 
    instance::{Instance, InstanceCreateInfo}, 
    memory::allocator::StandardMemoryAllocator, 
    sync::{
        self,
        GpuFuture, 
    }
};

use crate::{
    macros::error_map, 
	mesh_data::mesh_data_internal::MeshDataInternal, 
	pipeline::pipeline_internal::PipelineInternal, 
	render_engine::{
        create_info::RenderThreadCreateInfo, 
		render_command::RenderEngineCommand, 
		render_resources::DefaultResources,
    }, 
	render_surface::RenderSurface, 
	renderable::Renderable, 
	shader::shader_internal::ShaderInternal, 
	texture::texture_internal::TextureInternal,
};

pub(crate) struct RenderThread {
    pub(super) command_channel: Receiver<RenderEngineCommand>,

	pub(crate) mesh_data: HashMap<Uuid, MeshDataInternal>,
    pub(crate) shaders: HashMap<Uuid, ShaderInternal>,
	pub(crate) pipelines: HashMap<Uuid, PipelineInternal>,
    pub(crate) textures: HashMap<Uuid, TextureInternal>,

	pub(crate) default_resources: DefaultResources,

    pub(crate) renderables: HashMap<Uuid, Box<dyn Renderable>>,

    pub(crate) render_surfaces: HashMap<Uuid, Box<dyn RenderSurface>>,

	pub(crate) instance: Arc<Instance>,
    pub(crate) device: Arc<Device>,
    pub(crate) graphics_queue: Arc<Queue>,
    pub(crate) graphics_future: Option<Box<dyn GpuFuture + Send>>, 
    pub(crate) transfer_queue: Arc<Queue>,
    pub(crate) transfer_future: Option<Box<dyn GpuFuture + Send>>,

	pub(crate) buffer_allocator: Arc<StandardMemoryAllocator>,
	pub(crate) command_allocator: Arc<StandardCommandBufferAllocator>,
	pub(crate) descriptor_allocator: Arc<StandardDescriptorSetAllocator>,

    pub(super) should_close: bool,
}

impl RenderThread {
    pub(super) fn new(create_info: RenderThreadCreateInfo, command_channel: Receiver<RenderEngineCommand>) -> Result<Self, ()> {
        let library = VulkanLibrary::new().map_err(error_map!())?;

        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                enabled_extensions: create_info.instance_extensions,

                application_name: create_info.app_name,
                application_version: create_info.app_vers,

                engine_name: create_info.eng_name,
                engine_version: create_info.eng_vers,

                ..Default::default()
            }
        ).map_err(error_map!())?;

        let physical_device = instance.enumerate_physical_devices().map_err(error_map!())?
            // Filter for devices that support requested device extensions
            .filter(|pd| pd.supported_extensions().contains(&create_info.device_extensions))
            // Filter for devices that support graphics operations
            .filter(|pd| pd.queue_family_properties().iter().find(|qf| qf.queue_flags.intersects(QueueFlags::GRAPHICS)).is_some())
            // Prioritize physical devices that are typically stronger
            .min_by_key(|pd| {
                match pd.properties().device_type {
                    PhysicalDeviceType::DiscreteGpu => 0,
                    PhysicalDeviceType::IntegratedGpu => 1,
                    PhysicalDeviceType::VirtualGpu => 2,
                    PhysicalDeviceType::Cpu => 3,
                    PhysicalDeviceType::Other => 4,
                    _ => 5,
                }
            }).ok_or(())?;

        let graphics_qf_index = physical_device.queue_family_properties().iter().enumerate()
            // Filter by queue families which support graphics operations
            .filter(|(_, qf)| qf.queue_flags.intersects(QueueFlags::GRAPHICS))
            // Select the one with the most available queues
            .max_by_key(|(_, qf)| qf.queue_count)
            .map(|(i, _)| i as u32).unwrap();

        let transfer_qf_index = physical_device.queue_family_properties().iter().enumerate()
            // Filter out the render queue index
            .filter(|(i, _)| *i as u32 != graphics_qf_index)
            // Filter by queue families which support transfer operations
            .filter(|(_, qf)| qf.queue_flags.intersects(QueueFlags::TRANSFER))
            // Select the one with the most available queues
            .max_by_key(|(_, qf)| qf.queue_count)
            .map(|(i, _)| i as u32)
            .unwrap_or(graphics_qf_index);

        let mut queue_create_infos = Vec::new();

        queue_create_infos.push(QueueCreateInfo {
            queue_family_index: graphics_qf_index,
            ..Default::default()
        });

        if transfer_qf_index != graphics_qf_index {
            queue_create_infos.push(QueueCreateInfo {
                queue_family_index: transfer_qf_index,
                ..Default::default()
            });
        }

        let (device, mut queues) = Device::new(
            physical_device,
            DeviceCreateInfo {
                queue_create_infos: queue_create_infos,
                enabled_extensions: create_info.device_extensions,
                enabled_features: create_info.device_features,
                ..Default::default()
            }
        ).map_err(error_map!())?;

        let graphics_queue = queues.next().unwrap();
        let transfer_queue = queues.next().unwrap_or(graphics_queue.clone());

		let buffer_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
		let command_allocator = Arc::new(StandardCommandBufferAllocator::new(device.clone(), Default::default()));
		let descriptor_allocator = Arc::new(StandardDescriptorSetAllocator::new(device.clone(), Default::default()));
		
		let default_resources = DefaultResources::generate(&device, &transfer_queue, &buffer_allocator, &command_allocator)?;

        Ok(RenderThread {
			command_channel:    	command_channel,

			mesh_data:				HashMap::new(),
			shaders:                HashMap::new(),
			pipelines:				HashMap::new(),
			textures:               HashMap::new(),

			default_resources:		default_resources,

			renderables:            HashMap::new(),

			render_surfaces:        HashMap::new(),

			instance:				instance.clone(),
			device:             	device.clone(),
			graphics_queue:     	graphics_queue,
			graphics_future:    	Some(sync::now(device.clone()).boxed_send()),
			transfer_queue:     	transfer_queue,
			transfer_future:    	Some(sync::now(device.clone()).boxed_send()),

			buffer_allocator: 		buffer_allocator,
			command_allocator: 		command_allocator,
			descriptor_allocator: 	descriptor_allocator,
			
			should_close:       	false,
		})
    }

    // boilerplate for later when multiple queues potentially exist
    pub(crate) fn get_graphics_queue(&mut self) -> Arc<Queue> {
        return self.graphics_queue.clone()
    }

    pub(crate) fn get_transfer_queue(&mut self) -> Arc<Queue> {
        return self.transfer_queue.clone()
    }
}
