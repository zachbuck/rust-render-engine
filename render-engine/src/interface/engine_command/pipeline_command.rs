
use std::sync::Arc;

use uuid::Uuid;

use crate::{
	engine_command::EngineCommand, 
	engine_future::channel_engine_future::ChannelEngineResponse, 
	shader::Shader,
};

#[derive(Debug)]
pub enum PipelineCommand {
	CreatePipeline {
		vertex_shader: 		Arc<Shader>,
		fragment_shader: 	Arc<Shader>,

		surfaces:			Box<[Uuid]>,

		response:			ChannelEngineResponse<Result<(Uuid,), ()>>,
	},

	DropPipeline {
		uuid: 				Uuid,
	}
}

impl Into<EngineCommand> for PipelineCommand {
	fn into(self) -> EngineCommand { 
		EngineCommand::PipelineCommand(Box::new(self)) 
	}
}
