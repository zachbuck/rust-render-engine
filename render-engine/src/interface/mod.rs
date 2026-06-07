
use std::{
	sync::mpsc::{Sender, channel},
	thread::Builder as ThreadBuilder,
};

use crate::{
	interface::{
		engine_command::EngineCommand,
		engine_future::EngineFuture, 
		engine_future::channel_engine_future::ChannelEngineFuture,
	}, 
	vulkan::render_engine::render_thread as v_render_thread
};

pub mod data_format;
pub mod engine_future;
pub mod instruction_buffer;
pub mod mesh_data;
pub mod pipeline;
pub mod render_target;
pub mod shader;
pub mod surface;

pub(crate) mod engine_command;

pub struct RenderEngine {
	pub backend: RenderingBackend,
	
	command_channel: Sender<EngineCommand>,
}

pub struct RenderEngineCreateInfo {
	pub app_name: String,
	pub app_vers: [u16; 3],
	pub backend: RenderingBackend,
}

#[derive(Clone, Copy)]
pub enum RenderingBackend {
	Vulkan,
}

impl RenderEngine {
	pub fn new(create_info: RenderEngineCreateInfo) -> Result<Self, ()> {
		let (sender, receiver) = channel();
		let (future, response) = ChannelEngineFuture::new();

		let engine = RenderEngine {
			command_channel: sender,
			backend: create_info.backend,
		};

		let thread = ThreadBuilder::new()
			.name("Render Thread".to_string());

		let result;
		match create_info.backend {
			RenderingBackend::Vulkan => {
				result = thread.spawn(move || v_render_thread!(create_info, receiver, response));
			},
		}
		result.map_err(|_| ())?;

		if let Err(()) = future.unwrap() { return Err(()) }

		Ok(engine)
	}
}

impl Drop for RenderEngine {
	fn drop(&mut self) {
		let _ = self.command_channel.send(EngineCommand::DropEngine);
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
