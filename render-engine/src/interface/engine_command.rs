
use uuid::Uuid;

use crate::{
	data_formats::Vertex3D, 
	engine_future::channel_engine_future::ChannelEngineResponse, 
	surface::window_surface::WindowSurfaceCreateInfo,
};

#[derive(Debug)]
pub enum EngineCommand {
	ProcessRenderInstructionBuffer {
		instructions: Box<[RenderInstruction]>,
		response: ChannelEngineResponse<Result<(), ()>>,
	},
	
	MeshDataCommand(Box<MeshDataCommand>),
	WindowSurfaceCommand(Box<WindowSurfaceCommand>),

	DropRenderThread,
}

#[derive(Debug)]
pub enum RenderInstruction {
	BeginRendering {
		uuid: Uuid,
	},
	EndRendering,
}

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

#[derive(Debug)]
pub enum WindowSurfaceCommand {
	CreateWindowSurface {
		create_info: WindowSurfaceCreateInfo,

		response:	ChannelEngineResponse<Result<(Uuid,), ()>>,
	},

	DropWindowSurface {
		uuid: 		Uuid,
	},
}

impl Into<EngineCommand> for MeshDataCommand {
	fn into(self) -> EngineCommand { EngineCommand::MeshDataCommand(Box::new(self)) }
}

impl Into<EngineCommand> for WindowSurfaceCommand {
	fn into(self) -> EngineCommand { EngineCommand::WindowSurfaceCommand(Box::new(self)) }
}

