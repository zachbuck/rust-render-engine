
use crate::interface::instruction_buffer::InstructionBuffer;

pub enum EngineCommand {
	DropEngine,
	InstructionBuffer(InstructionBuffer),

	ImageSurfaceCommand(Box<ImageSurfaceCommand>),
	MeshDataCommand(Box<MeshDataCommand>),
	PipelineCommand(Box<PipelineCommand>),
	ShaderCommand(Box<ShaderCommand>),
	RenderObjectCommand(Box<RenderObjectCommand>),
}

pub enum ImageSurfaceCommand {
	CreateImageSurface,
	DropImageSurface,
}

pub enum MeshDataCommand {
	CreateMeshData,
	DropMeshData,
}

pub enum PipelineCommand {
	CreatePipeline,
	DropPipeline,
}

pub enum ShaderCommand {
	CreateShader,
	DropShader,
}

pub enum RenderObjectCommand {
	CreateRenderObject,
	DropRenderObject,
}
