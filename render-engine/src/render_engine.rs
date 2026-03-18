use std::{
    sync::{
        Arc, 
        mpsc::{Receiver, Sender, TryRecvError, channel},
    }, 
    thread::Builder as ThreadBuilder
};

use vulkano::{
    Version, 
    VulkanLibrary, 
    device::{
        Device, 
        DeviceCreateInfo, 
        DeviceExtensions, 
        DeviceFeatures, 
        Queue, 
        QueueCreateInfo, 
        QueueFlags, 
        physical::PhysicalDeviceType
    }, 
    instance::{Instance, InstanceCreateInfo, InstanceExtensions}, 
    sync::{self, GpuFuture}
};

use crate::mesh_data::MeshDataCommand;

pub struct RenderEngine {
    pub(crate) command_channel: Sender<RenderEngineCommand>,
}

macro_rules! run_render_thread {
    ($create_info: ident, $command_channel: ident, $init_channel: ident) => {
        move || {
            let result = RenderThread::new($create_info, $command_channel);
            if result.is_err() {
                $init_channel.send(Err(unsafe { result.unwrap_err_unchecked() })).unwrap();
                return
            }
            $init_channel.send(Ok(())).unwrap();

            let mut internal = result.unwrap();

            while !internal.should_close {
                internal.process_command();
            }
        }
    };
}

impl RenderEngine {
    pub fn new(create_info: RenderEngineCreateInfo) -> Result<Arc<Self>, ()> {
        let (command_s, command_r) = channel();
        let (init_s, init_r) = channel();

        ThreadBuilder::new()
            .name("Render Thread".to_string())
            .spawn(run_render_thread!(create_info, command_r, init_s))
            .map_err(|_| ())?;

        init_r.recv().unwrap()?;

        Ok(Arc::new(RenderEngine {
            command_channel: command_s,
        }))
    }
}

impl Drop for RenderEngine {
    fn drop(&mut self) {
        self.command_channel.send(RenderEngineCommand::Exit).unwrap();
    }
}

pub(crate) struct RenderThread {
    command_channel: Receiver<RenderEngineCommand>,

    device: Arc<Device>,
    graphics_queue: Arc<Queue>,
    graphics_future: Option<Box<dyn GpuFuture>>, 
    transfer_queue: Arc<Queue>,
    transfer_future: Option<Box<dyn GpuFuture>>,

    should_close: bool,
}

impl RenderThread {
    fn new(create_info: RenderEngineCreateInfo, command_channel: Receiver<RenderEngineCommand>) -> Result<Self, ()> {
        let library = VulkanLibrary::new().map_err(|_| ())?;

        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                enabled_extensions: create_info.instance_extensions,

                application_name: create_info.app_name,
                application_version: create_info.app_vers,

                engine_name: RenderEngineCreateInfo::get_eng_name(),
                engine_version: RenderEngineCreateInfo::get_eng_vers(),

                ..Default::default()
            }
        ).map_err(|_| ())?;

        let physical_device = instance.enumerate_physical_devices().map_err(|_| ())?
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
        ).map_err(|_| ())?;

        let graphics_queue = queues.next().unwrap();
        let transfer_queue = queues.next().unwrap_or(graphics_queue.clone());

        Ok(RenderThread {
            command_channel:    command_channel,

            device:             device.clone(),
            graphics_queue:     graphics_queue,
            graphics_future:    Some(sync::now(device.clone()).boxed()),
            transfer_queue:     transfer_queue,
            transfer_future:    Some(sync::now(device.clone()).boxed()),

            should_close:       false,
        })
    }

    fn process_command(&mut self) {
        let result = self.command_channel.try_recv();
        let command;
        match result {
            Ok(rec) => command = rec,
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Disconnected) => {self.process_exit(); return},
        }

        match command {
            RenderEngineCommand::Exit => self.process_exit(),
            RenderEngineCommand::MeshDataCommand(command) => self.process_mesh_data_command(command),
        }
    }

    fn process_exit(&mut self) {

    }
}

pub struct RenderEngineCreateInfo {
    app_name: Option<String>,
    app_vers: Version,

    instance_extensions: InstanceExtensions,
    device_extensions: DeviceExtensions,
    device_features: DeviceFeatures,
}

impl RenderEngineCreateInfo {
    pub fn new() -> Self {
        RenderEngineCreateInfo {
            app_name: None,
            app_vers: Version { major: 0, minor: 1, patch: 0 },

            instance_extensions: InstanceExtensions::empty(),
            device_extensions: DeviceExtensions::empty(),
            device_features: DeviceFeatures::empty(),
        }
    }

    pub fn with_app_name(mut self, app_name: String) -> Self { self.app_name = Some(app_name); self }
    pub fn with_app_vers(mut self, major: u32, minor: u32, patch: u32) -> Self { self.app_vers = Self::get_version(major, minor, patch); self }

    fn get_eng_name() -> Option<String> { Some(env!("CARGO_PKG_NAME").to_string()) }
    fn get_eng_vers() -> Version { Self::get_version(env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap(), env!("CARGO_PKG_VERSION_MINOR").parse().unwrap(), env!("CARGO_PKG_VERSION_PATCH").parse().unwrap()) }

    fn get_version(major: u32, minor: u32, patch: u32) -> Version { Version { major, minor, patch } }
}

pub(crate) enum RenderEngineCommand {
    Exit,
    MeshDataCommand(MeshDataCommand),
}

pub struct EngineFuture<T> {
    channel: Receiver<T>,
}

impl<T> EngineFuture<T> {
    pub fn try_unwrap(self) -> Result<T, ()> { self.channel.try_recv().map_err(|_| ()) }
    pub fn unwrap(self) -> T { self.channel.recv().unwrap() }

    pub(crate) fn new(channel: Receiver<T>) -> Self { EngineFuture { channel } }
}
