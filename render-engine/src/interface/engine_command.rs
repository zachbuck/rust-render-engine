
use uuid::Uuid;

use crate::interface::{
	data_format::{
		index::IndexCollection,
		vertex::VertexCollection,
	},
	engine_future::channel_engine_future::ChannelEngineResponse,
	instruction_buffer::InstructionBuffer,
};

pub enum EngineCommand {
	DropEngine,
	InstructionBufferCommand(Box<InstructionBufferCommand>),

	ImageSurfaceCommand(Box<ImageSurfaceCommand>),
	MeshDataCommand(Box<MeshDataCommand>),
	PipelineCommand(Box<PipelineCommand>),
	ShaderCommand(Box<ShaderCommand>),
	RenderObjectCommand(Box<RenderObjectCommand>),
}

pub enum ImageSurfaceCommand {
	CreateImageSurface {
		dimensions: [u32; 2],
		vulkan_format: vulkano::format::Format,

		response: ChannelEngineResponse<Result<(Uuid,), ()>>,
	},
	DropImageSurface {
		uuid: Uuid,
	},
}

pub enum InstructionBufferCommand {
	OneTimeSubmit {
		instructions: InstructionBuffer,

		response: ChannelEngineResponse<Result<(), ()>>,
	}
}

pub enum MeshDataCommand {
	CreateMeshData {
		vertices: VertexCollection,
		indices: IndexCollection,

		response: ChannelEngineResponse<Result<(Uuid,), ()>>,
	},
	DropMeshData {
		uuid: Uuid,
	}
}

pub enum PipelineCommand {
	CreatePipeline {
		surface: Uuid,
		shaders: Box<[Uuid]>,

		response: ChannelEngineResponse<Result<(Uuid,), ()>>,
	},
	DropPipeline {
		uuid: Uuid,
	},
}

pub enum ShaderCommand {
	CreateShaderSpirv {
		binary: Box<[u32]>,

		response: ChannelEngineResponse<Result<(Uuid,), ()>>,
	},
	DropShader {
		uuid: Uuid,
	},
}

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
