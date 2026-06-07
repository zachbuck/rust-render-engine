
use std::sync::{Arc, mpsc::Sender};

use uuid::Uuid;

use crate::interface::{
	engine_command::{EngineCommand, PipelineCommand},
	engine_future::{
		EngineFuture,
		channel_engine_future::ChannelEngineFuture,
		transform_engine_future::TransformEngineFuture,
	}, 
	shader::{Shader, ShaderStage},
	surface::Surface,
};

pub struct GraphicsPipeline {
	pub surface: Arc<dyn Surface>,
	pub stages: ShaderStages,
	pub shaders: Box<[Arc<Shader>]>,

	pub(crate) uuid: Uuid,
	command_channel: Sender<EngineCommand>,
}

pub struct ShaderStages {
	stages: u32,
}

impl GraphicsPipeline {
	pub fn new<T>(surface: &Arc<T>, shaders: Box<[Arc<Shader>]>) -> impl EngineFuture<Result<Arc<Self>, ()>> 
	where T: Surface + 'static {
		let mut stages = ShaderStages::empty();
		let mut uuids = Vec::with_capacity(shaders.len());
		for shader in &shaders {
			stages.add(shader.stage);
			uuids.push(shader.uuid);
		}

		let command_channel = surface.get_command_channel().clone();
		let surface_copy = surface.clone();
		let (future, response) = TransformEngineFuture::new(
			ChannelEngineFuture::new(), 
			Box::new(|result: Result<_, _>| result.map(|(uuid,)| {
				Arc::new(GraphicsPipeline { surface: surface_copy, uuid, stages, shaders, command_channel })
			})),
		);

		let command = PipelineCommand::CreatePipeline { surface: *surface.get_uuid(), shaders: uuids.into_boxed_slice(), response };
		let command = EngineCommand::PipelineCommand(Box::new(command));
		let _ = surface.get_command_channel().send(command);

		return future
	}
}

impl Drop for GraphicsPipeline {
	fn drop(&mut self) {
		let command = PipelineCommand::DropPipeline { uuid: self.uuid };
		let command = EngineCommand::PipelineCommand(Box::new(command));
		let _ = self.command_channel.send(command);
	}
}

impl ShaderStages {
	pub fn empty() -> Self { ShaderStages { stages: 0x0000 } }
	pub fn add(&mut self, stage: ShaderStage) { self.stages &= stage as u32; }
	pub fn has_stage(&self, stage: ShaderStage) -> bool { self.stages & stage as u32 != 0 }
}
