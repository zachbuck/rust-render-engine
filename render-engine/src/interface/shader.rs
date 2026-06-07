
use std::sync::{Arc, mpsc::Sender};

use uuid::Uuid;

use crate::interface::{
	RenderEngine,
	engine_command::{EngineCommand, ShaderCommand},
	engine_future::{
		EngineFuture,
		channel_engine_future::ChannelEngineFuture,
		transform_engine_future::TransformEngineFuture,
	},
};

pub struct Shader {
	pub stage: ShaderStage,
	
	pub(crate) uuid: Uuid,
	command_channel: Sender<EngineCommand>,
}

#[derive(Clone, Copy)]
#[repr(u32)]
pub enum ShaderStage {
	Vertex = 	0x0001,
	Fragment = 	0x0002,
}

impl Shader {
	pub fn from_spirv(render_engine: &RenderEngine, binary: Box<[u32]>) -> impl EngineFuture<Result<Arc<Self>, ()>> {
		let command_channel = render_engine.command_channel.clone();
		let (future, response) = TransformEngineFuture::new(
			ChannelEngineFuture::new(), 
			Box::new(|result: Result<_, _>| result.map(|(uuid,)|
				Arc::new(Shader { uuid, command_channel, stage: ShaderStage::Vertex })
			))
		);

		let command = ShaderCommand::CreateShaderSpirv {
			binary,
			response,
		};
		let command = EngineCommand::ShaderCommand(Box::new(command));
		let _ = render_engine.command_channel.send(command);

		return future;
	}
}

impl Drop for Shader {
	fn drop(&mut self) {
		let command = ShaderCommand::DropShader {
			uuid: self.uuid,
		};
		let command = EngineCommand::ShaderCommand(Box::new(command));
		let _ = self.command_channel.send(command);
	}
}
