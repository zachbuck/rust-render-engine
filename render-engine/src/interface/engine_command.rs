
use uuid::Uuid;

use crate::{
	data_formats::Vertex3D, 
	engine_future::channel_engine_future::ChannelEngineResponse, 
	surface::{
		RenderPassCreateInfo, 
		window_surface::WindowSurfaceCreateInfo,
	},
};

#[derive(Debug)]
pub enum EngineCommand {
	ProcessRenderInstructionBuffer {
		instructions: Box<[RenderInstruction]>,
		response: ChannelEngineResponse<Result<(), ()>>,
	},
	
	MeshDataCommand(Box<MeshDataCommand>),
	#[expect(unused)]
	ShaderCommand(Box<ShaderCommand>),
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

impl Into<EngineCommand> for MeshDataCommand {
	fn into(self) -> EngineCommand { EngineCommand::MeshDataCommand(Box::new(self)) }
}

#[derive(Debug)]
pub enum ShaderCommand {
	CreateShader {
		source: 	Box<[u32]>,

		response:	ChannelEngineResponse<Result<(Uuid,), ()>>
	},

	DropShader {
		uuid:		Uuid,
	}
}

impl Into<EngineCommand> for ShaderCommand {
	fn into(self) -> EngineCommand { EngineCommand::ShaderCommand(Box::new(self)) }
}

#[derive(Debug)]
pub enum WindowSurfaceCommand {
	CreateWindowSurface {
		create_info: 		WindowSurfaceCreateInfo,
		#[expect(unused)]
		render_pass_info: 	RenderPassCreateInfo,

		response:			ChannelEngineResponse<Result<(Uuid,), ()>>,
	},

	DropWindowSurface {
		uuid: 				Uuid,
	},
}

impl Into<EngineCommand> for WindowSurfaceCommand {
	fn into(self) -> EngineCommand { EngineCommand::WindowSurfaceCommand(Box::new(self)) }
}
