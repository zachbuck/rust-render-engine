
use std::{
	collections::HashSet, 
	sync::{
		Arc, 
		mpsc::sync_channel
	}
};

use uuid::Uuid;

use crate::{
	pipeline::pipeline_command::PipelineCommand, 
	render_engine::{
		RenderEngine, 
		engine_future::EngineFuture, 
		render_command::RenderEngineCommand
	}, 
	shader::{Shader, ShaderType}
};

pub(crate) mod pipeline_command;
pub(crate) mod pipeline_internal;

#[derive(Debug)]
pub struct Pipeline {
	uuid: Uuid,
	render_engine: Arc<RenderEngine>,

	pub shaders: Box<[Arc<Shader>]>,
}

impl Pipeline {
	pub fn new(render_engine: Arc<RenderEngine>, shaders: &[Arc<Shader>]) -> EngineFuture<Result<Arc<Pipeline>, ()>> {
		// Do some error checking on the input to make sure that this will link into a valid GraphicsPipeline
		// 1. Only one of each stage
		// 2. Must contain Vertex Shader
		// 3. ... (more requirements but they come when more shader types implemented)

		let mut stages = HashSet::new();
		for shader in shaders {
			if stages.contains(&shader.shader_type) {
				return EngineFuture::new_immediate(Err(()));
			}

			stages.insert(shader.shader_type);
		}

		if !stages.contains(&ShaderType::Vertex) {
			return EngineFuture::new_immediate(Err(()));
		}

		let (send, recv) = sync_channel(1);

		let shaders = shaders.to_owned().into_boxed_slice();

		render_engine.command_channel.send(
			PipelineCommand::CreatePipeline { 
				sender: send, 
				shaders: shaders,
				engine: render_engine.clone(),
			}.into()
		).unwrap();

		return EngineFuture::new_single(recv);
	}
}

impl Drop for Pipeline {
	fn drop(&mut self) {
		self.render_engine.command_channel.send(
			PipelineCommand::DropPipeline { 
				uuid: self.uuid 
			}.into()
		).unwrap()
	}
}
