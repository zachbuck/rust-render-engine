
use std::sync::Arc;

use parsing::spir_v::shader::SpirvShader;
use uuid::Uuid;

use crate::{
	data_formats::Vertex3D, 
	engine_future::channel_engine_future::ChannelEngineResponse, 
	shader::Shader, 
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
	PipelineCommand(Box<PipelineCommand>),
	RenderObjectCommand(Box<RenderObjectCommand>),
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

	RenderObject {
		uuid: Uuid,
	}
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
	fn into(self) -> EngineCommand { EngineCommand::PipelineCommand(Box::new(self)) }
}

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
	fn into(self) -> EngineCommand { EngineCommand::RenderObjectCommand(Box::new(self)) }
}

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

#[derive(Debug)]
pub enum WindowSurfaceCommand {
	CreateWindowSurface {
		create_info: 		WindowSurfaceCreateInfo,
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
