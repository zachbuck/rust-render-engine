
use uuid::Uuid;

use crate::{
	data_formats::Vertex3D, 
	engine_command::EngineCommand,
	engine_future::channel_engine_future::ChannelEngineResponse,
};

#[derive(Debug)]
pub enum MeshDataCommand {
	CreateMeshData {
		vertices: 	Box<[Vertex3D]>,
		indices:	Box<[u32]>,

		response: 	ChannelEngineResponse<Result<(Uuid,), ()>>,
	},

	DropMeshData {
		uuid:		Uuid,
	},
}

impl Into<EngineCommand> for MeshDataCommand {
	fn into(self) -> EngineCommand { 
		EngineCommand::MeshDataCommand(Box::new(self)) 
	}
}
