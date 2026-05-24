
use std::sync::mpsc::Sender;

use crate::interface::engine_commands::EngineCommand;

pub mod data_format;
pub mod engine_future;
pub mod instruction_buffer;
pub mod mesh_data;
pub mod pipeline;
pub mod render_target;
pub mod shader;
pub mod surface;

pub(crate) mod engine_commands;

pub struct RenderEngine {
	command_channel: Sender<EngineCommand>
}

pub struct RenderEngineCreateInfo {
	pub app_name: String,
	pub app_vers: [u16; 3],
	pub backend: RenderingBackend,
}

pub enum RenderingBackend {
	Vulkan,
}

impl RenderEngine {
	pub fn new(create_info: RenderEngineCreateInfo) -> Result<Self, ()> {
		todo!()
	}
}

impl Drop for RenderEngine {
	fn drop(&mut self) {
		todo!()
	}
}

impl RenderEngineCreateInfo {
	pub fn default(backend: RenderingBackend) -> Self {
		RenderEngineCreateInfo {
			app_name: "My App".to_string(),
			app_vers: [0, 1, 0],
			backend,
		}
	}
}
