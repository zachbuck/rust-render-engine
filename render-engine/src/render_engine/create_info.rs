
use sdl2::Sdl;
use shaderc::Compiler;
use vulkano::{
	Version, 
	device::{DeviceExtensions, DeviceFeatures}, 
	instance::InstanceExtensions, 
	swapchain::Surface,
};

use crate::render_engine::SdlResources;

/// Flags for the creation of a `RenderEngine`
pub struct RenderEngineFlags {
	/// Controls if a SPIR-V compiler is initizlized with the `RenderEngine`.
	pub feature_spirv_compiler: bool,

	/// Controls if features related to windowing are available.
	pub feature_windowing: bool,
	/// The size of the buffer of each window for events that are polled by the `RenderEngine`.
	/// 
	/// Any events that overflow the buffer will be dropped.
	/// 
	/// Defaults to `32`.
	pub event_buffer_size: u32,
}

impl RenderEngineFlags {
	pub fn empty() -> Self {
		RenderEngineFlags { 
			feature_spirv_compiler: false, 
			feature_windowing: false,
			event_buffer_size: 32,
		}
	}

	pub(super) fn generate_spirv_compiler(&self) -> Option<Compiler> {
		if self.feature_spirv_compiler {
			return Some(Compiler::new().unwrap());
		}
		return None;
	}

	pub(super) fn generate_sdl(&self) -> Option<Sdl> {
		if self.feature_windowing {
			return Some(sdl2::init().unwrap());
		}
		return None;
	}
}

impl Default for RenderEngineFlags {
	fn default() -> Self {
		Self::empty()
	}
}

#[inline]
pub(crate) fn get_eng_name() -> &'static str {
	env!("CARGO_PKG_NAME")
}

#[inline]
pub(crate) fn get_eng_version() -> Version {
	let major = env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap();
	let minor = env!("CARGO_PKG_VERSION_MINOR").parse().unwrap();
	let patch = env!("CARGO_PKG_VERSION_PATCH").parse().unwrap();

	get_version(major, minor, patch)
}

#[inline]
pub(crate) fn get_version(major: u32, minor: u32, patch: u32) -> Version {
	Version { major, minor, patch }
}

pub(super) struct RenderThreadCreateInfo {
	pub(super) app_name: Option<String>,
	pub(super) app_vers: Version,

	pub(super) eng_name: Option<String>,
	pub(super) eng_vers: Version,

	pub(super) instance_extensions: InstanceExtensions,
	pub(super) device_extensions: DeviceExtensions,
	pub(super) device_features: DeviceFeatures,
}

impl RenderThreadCreateInfo {
	pub(super) fn new(app_name: &str, app_vers: &[u32; 3], flags: &RenderEngineFlags, sdl: &Option<SdlResources>) -> Self {
		let mut instance_extensions = InstanceExtensions::empty();
		let mut device_extensions = DeviceExtensions::empty();
		let mut device_features = DeviceFeatures::empty();

		// Needed in all cases.
		device_extensions.khr_dynamic_rendering = true;
		device_features.dynamic_rendering = true;

		// features_spirv_compiler 
		// does not need any changes.

		// features_windowing
		if flags.feature_windowing {
			let video_subsytem = &sdl.as_ref().unwrap().video;
			let window = video_subsytem.window("Render Engine", 1, 1)
				.vulkan()
				.hidden()
				.build().unwrap();
			instance_extensions = instance_extensions.union(&Surface::required_extensions(&window).unwrap());
			drop(window);

			device_extensions.khr_swapchain = true;
		}

		RenderThreadCreateInfo {
			app_name: Some(app_name.to_string()),
			app_vers: get_version(app_vers[0], app_vers[1], app_vers[2]),

			eng_name: Some(get_eng_name().to_string()),
			eng_vers: get_eng_version(),

			instance_extensions: instance_extensions,
			device_extensions: device_extensions,
			device_features: device_features,
		}
	}
}
