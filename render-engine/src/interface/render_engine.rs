
use std::{
	sync::{
		Arc, 
		mpsc::{Sender, channel}
	}, 
	thread::Builder as ThreadBuilder,
};

use crate::{
	engine_command::EngineCommand, 
	engine_future::{
		EngineFuture, 
		channel_engine_future::ChannelEngineFuture,
	}, 
	render_instruction_buffer::RenderInstructionBuffer, 
	vulkan::render_thread::start_vulkan_render_thread,
};

pub struct RenderEngine {
	pub(crate) command_channel: Sender<EngineCommand>,
}

pub struct RenderEngineCreateInfo {
	pub backend: 						RenderEngineBackend,

	pub app_name: 						String,
	pub app_version: 					[u32; 3],
}

pub enum RenderEngineBackend {
	Vulkan,
}

impl RenderEngine {
	pub fn new(create_info: RenderEngineCreateInfo) -> Result<Arc<Self>, ()> {
		let (sender, receiver) = channel();
		let (future, response) = ChannelEngineFuture::new();

		let _ = ThreadBuilder::new()
			.name("Render Thread".to_string())
			.spawn(move || start_vulkan_render_thread!(create_info, receiver, response))
			.map_err(|_| ())?;

		let result = future.wait();
		if let Err(e) = result { return Err(e) }

		Ok(Arc::new(RenderEngine {
			command_channel: sender,
		}))
	}

	pub fn submit_render_instructions(&self, buffer: RenderInstructionBuffer) -> impl EngineFuture<Result<(), ()>> {
		let (future, response) = ChannelEngineFuture::new();

		let _ = self.command_channel.send(EngineCommand::ProcessRenderInstructionBuffer { 
			instructions: 	buffer.buffer, 
			response: 		response,
		});

		return future;
	}
}

impl Drop for RenderEngine {
	fn drop(&mut self) {
		let _ = self.command_channel.send(EngineCommand::DropRenderThread);
	}
}

impl RenderEngineCreateInfo {
	pub fn with_backend(backend: RenderEngineBackend) -> Self {
		RenderEngineCreateInfo {
			backend: backend,
			app_name: "My App".to_string(),
			app_version: [0, 1, 0],
		}
	}
}
