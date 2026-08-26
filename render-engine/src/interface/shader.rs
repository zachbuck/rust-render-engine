
use std::sync::{
	Arc, 
	mpsc::Sender,
};

use uuid::Uuid;

use crate::{
	engine_command::{EngineCommand, ShaderCommand}, 
	engine_future::EngineFuture, 
	render_engine::RenderEngine,
};

pub struct Shader {
	uuid: 				Uuid,
	command_channel: 	Sender<EngineCommand>,
}

impl Shader {
	// fn new(render_engine: &Arc<RenderEngine>, shader_binary: Box<[u32]>) -> impl EngineFuture<Result<Arc<Shader>, ()>> {

	// }
}

impl Drop for Shader {
	fn drop(&mut self) {
		let _ = self.command_channel.send(ShaderCommand::DropShader {
			uuid: self.uuid,
		}.into());
	}
}
