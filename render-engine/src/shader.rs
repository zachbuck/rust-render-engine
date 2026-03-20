
use std::sync::{
	Arc, 
	mpsc::sync_channel
};

use shaderc::ShaderKind;
use uuid::Uuid;
use vulkano::shader::{
	spirv::ExecutionModel
};

use crate::{
	render_engine::{
		RenderEngine, 
		engine_future::EngineFuture, 
		render_command::RenderEngineCommand
	}, 
	shader::shader_command::ShaderCommand
};

pub(crate) mod shader_internal;
pub(crate) mod shader_command;

#[derive(Debug)]
pub struct Shader {
	uuid: Uuid,
	render_engine: Arc<RenderEngine>,

	pub shader_type: ShaderType,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
#[derive(Hash)]
#[derive(Clone, Copy)]
#[derive(Debug)]
pub enum ShaderType {
	Vertex,
	Fragment,
}

impl Shader { 
	pub fn compile(render_engine: Arc<RenderEngine>, shader_name: &str, shader_type: ShaderType, shader_source: &str) -> Result<Box<[u32]>, ()> {
		let compiler = render_engine.spirv_compiler.as_ref().ok_or(())?;

		let artifact = compiler.compile_into_spirv(
			shader_source, 
			shader_type.into(), 
			shader_name, 
			"main", 
			None
		).map_err(|_| ())?;

		return Ok(artifact.as_binary().to_owned().into_boxed_slice());
	}

	pub fn new(render_engine: Arc<RenderEngine>, binary: Box<[u32]>) -> EngineFuture<Result<Arc<Self>, ()>> {
		let (send, recv) = sync_channel(1);

		render_engine.command_channel.send(
			RenderEngineCommand::ShaderCommand(
				ShaderCommand::CreateShader { 
					sender: send, 
					binary, 
					engine: render_engine.clone(), 
				}
			)
		).unwrap();

		return EngineFuture::new(recv);
	}
}

impl Drop for Shader {
	fn drop(&mut self) {
		self.render_engine.command_channel.send(
			RenderEngineCommand::ShaderCommand(
				ShaderCommand::DropShader { 
					uuid: self.uuid,
				}
			)
		).unwrap();
	}
}

impl Into<ShaderKind> for ShaderType {
	fn into(self) -> ShaderKind {
		match self {
			ShaderType::Vertex 		=> ShaderKind::Vertex,
			ShaderType::Fragment 	=> ShaderKind::Fragment,
		}
	}
}

impl From<ExecutionModel> for ShaderType {
	fn from(value: ExecutionModel) -> Self {
		match value {
			ExecutionModel::Vertex		=> ShaderType::Vertex,
			ExecutionModel::Fragment 	=> ShaderType::Fragment,
			_ => panic!("Unknown ExecutionModel type: '{:?}'", value),
		}
	}
}
