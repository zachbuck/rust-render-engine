
use parsing::spir_v::shader::SpirvShader;
use uuid::Uuid;

use crate::{
	engine_command::EngineCommand,
	engine_future::channel_engine_future::ChannelEngineResponse,
};

#[derive(Debug)]
pub enum ShaderCommand {
	CreateShader {
		source: 	SpirvShader,

		response:	ChannelEngineResponse<Result<(Uuid,), ()>>
	},

	DropShader {
		uuid:		Uuid,
	}
}

impl Into<EngineCommand> for ShaderCommand {
	fn into(self) -> EngineCommand { EngineCommand::ShaderCommand(Box::new(self)) }
}
