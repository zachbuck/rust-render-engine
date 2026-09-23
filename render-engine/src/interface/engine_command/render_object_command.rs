
use uuid::Uuid;

use crate::{
	engine_command::EngineCommand,
	engine_future::channel_engine_future::ChannelEngineResponse,
};

#[derive(Debug)]
pub enum RenderObjectCommand {
	CreateRenderObject {
		mesh_data: Uuid,
		pipeline: Uuid,

		response: ChannelEngineResponse<Result<(Uuid,), ()>>,
	},

	DropRenderObject {
		uuid: Uuid,
	},
}

impl Into<EngineCommand> for RenderObjectCommand {
	fn into(self) -> EngineCommand { 
		EngineCommand::RenderObjectCommand(Box::new(self)) 
	}
}
