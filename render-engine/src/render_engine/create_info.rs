
use vulkano::{
    Version, 
    device::{DeviceExtensions, DeviceFeatures}, 
    instance::InstanceExtensions
};

#[derive(Debug)]
pub struct RenderEngineCreateInfo {
    pub(super) app_name: Option<String>,
    pub(super) app_vers: Version,

    pub(super) instance_extensions: InstanceExtensions,
    pub(super) device_extensions: DeviceExtensions,
    pub(super) device_features: DeviceFeatures,

    pub(super) flags: u64,
}

#[repr(u64)]
#[derive(Debug)]
pub(super) enum RenderEngineCreateInfoFlags {
    InitSpirvCompiler = 0x0000_0001,
}

impl RenderEngineCreateInfo {
    pub fn new() -> Self {
        RenderEngineCreateInfo {
            app_name: None,
            app_vers: Version { major: 0, minor: 1, patch: 0 },

            instance_extensions: InstanceExtensions {
				..InstanceExtensions::empty()
			},
            device_extensions: DeviceExtensions {
				..DeviceExtensions::empty()
			},
            device_features: DeviceFeatures {
				dynamic_rendering: true,
				..DeviceFeatures::empty()
			},

            flags: 0x0000_0000 
        }
    }

    pub fn with_app_name(mut self, app_name: String) -> Self { self.app_name = Some(app_name); self }
    pub fn with_app_vers(mut self, major: u32, minor: u32, patch: u32) -> Self { self.app_vers = Self::get_version(major, minor, patch); self }

    pub fn with_spirv_compiler(mut self) -> Self { self.flags |= RenderEngineCreateInfoFlags::InitSpirvCompiler as u64; self }

    pub(super) fn get_eng_name() -> Option<String> { Some(env!("CARGO_PKG_NAME").to_string()) }
    pub(super) fn get_eng_vers() -> Version { Self::get_version(env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap(), env!("CARGO_PKG_VERSION_MINOR").parse().unwrap(), env!("CARGO_PKG_VERSION_PATCH").parse().unwrap()) }

    fn get_version(major: u32, minor: u32, patch: u32) -> Version { Version { major, minor, patch } }
}
