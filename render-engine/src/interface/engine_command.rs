
use std::sync::Arc;

use crate::{
	engine_command::{
		mesh_data_command::MeshDataCommand,
		render_instruction::RenderInstruction,
		pipeline_command::PipelineCommand,
		render_object_command::RenderObjectCommand,
		shader_command::ShaderCommand,
		window_surface_command::WindowSurfaceCommand,
	}, 
	engine_future::channel_engine_future::ChannelEngineResponse, 
};

pub mod mesh_data_command;
pub mod render_instruction;
pub mod render_object_command;
pub mod pipeline_command;
pub mod shader_command;
pub mod window_surface_command;

#[derive(Debug)]
pub enum EngineCommand {
	ProcessRenderInstructionBuffer {
		instructions: Arc<[RenderInstruction]>,
		response: ChannelEngineResponse<Result<(), ()>>,
	},
	
	MeshDataCommand(Box<MeshDataCommand>),
	PipelineCommand(Box<PipelineCommand>),
	RenderObjectCommand(Box<RenderObjectCommand>),
	ShaderCommand(Box<ShaderCommand>),
	WindowSurfaceCommand(Box<WindowSurfaceCommand>),

	DropRenderThread,
}
