
use std::sync::{
	Arc, 
	mpsc::Sender,
};

use spir_v::shader::{ShaderStage, SpirvShader};
use uuid::Uuid;

use crate::{
	engine_command::{EngineCommand, ShaderCommand}, 
	engine_future::{
		EngineFuture, 
		channel_engine_future::ChannelEngineFuture, 
		then_transform_future::ThenTransformFuture
	}, 
	render_engine::RenderEngine, 
};

pub struct Shader {
	pub stage:			ShaderStage,

	uuid: 				Uuid,
	command_channel: 	Sender<EngineCommand>,
}

impl Shader {
	pub fn new(render_engine: &Arc<RenderEngine>, shader: SpirvShader) -> impl EngineFuture<Result<Arc<Shader>, ()>> {
		let command_channel = render_engine.command_channel.clone();
		let stage = *shader.get_stage();
		let (future, response) = ThenTransformFuture::new(
			ChannelEngineFuture::new(), 
			Box::new(move |result: Result<_, _>| result.map(
				|(uuid,)| Arc::new(Shader {
					stage: stage,
					uuid: uuid,
					command_channel: command_channel,
				})
			))
		);

		let _ = render_engine.command_channel.send(ShaderCommand::CreateShader { 
			source: shader, 
			response,
		}.into());

		return future;
	}
}

impl Drop for Shader {
	fn drop(&mut self) {
		let _ = self.command_channel.send(ShaderCommand::DropShader {
			uuid: self.uuid,
		}.into());
	}
}
