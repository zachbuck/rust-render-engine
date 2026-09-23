
use std::sync::{
	Arc, 
	mpsc::Sender,
};

use parsing::spir_v::shader::ShaderStage;
use uuid::Uuid;

use crate::{
	engine_command::{
		EngineCommand, 
		pipeline_command::PipelineCommand,
	}, 
	engine_future::{
		EngineFuture, 
		channel_engine_future::ChannelEngineFuture, 
		now_engine_future::NowEngineFuture, 
		then_transform_future::ThenTransformFuture
	}, 
	render_engine::RenderEngine, 
	shader::Shader, 
	surface::Surface,
};

pub struct Pipeline {
	pub(crate) uuid:		Uuid,
	command_channel:		Sender<EngineCommand>,

	pub vertex_shader: 		Arc<Shader>,
	pub fragment_shader: 	Arc<Shader>,
}

#[derive(Clone)]
pub struct PipelineCreateInfo<'a> {
	pub vertex_shader: 		Arc<Shader>,
	pub fragment_shader: 	Arc<Shader>,

	pub surfaces:			&'a[&'a dyn Surface],
}

impl Pipeline {
	pub fn new(render_engine: &Arc<RenderEngine>, create_info: PipelineCreateInfo) -> impl EngineFuture<Result<Arc<Self>, ()>> {
		let info_copy = create_info.clone();
		let command_channel = render_engine.command_channel.clone();

		if *create_info.vertex_shader.get_stage() != ShaderStage::Vertex { return Box::new(NowEngineFuture::new(Err(()))) as Box<dyn EngineFuture<_>> }
		if *create_info.fragment_shader.get_stage() != ShaderStage::Fragment { return Box::new(NowEngineFuture::new(Err(()))) as Box<dyn EngineFuture<_>> }

		let vertex_outputs = &create_info.vertex_shader.get_outputs()[1..];
		let fragment_inputs = create_info.fragment_shader.get_inputs();
		if vertex_outputs.len() != fragment_inputs.len() { return Box::new(NowEngineFuture::new(Err(()))) as Box<dyn EngineFuture<_>> }
		for x in 0..vertex_outputs.len() {
			if vertex_outputs[x] != fragment_inputs[x] { return Box::new(NowEngineFuture::new(Err(()))) as Box<dyn EngineFuture<_>> }
		}

		let (future, response) = ThenTransformFuture::new(
			ChannelEngineFuture::new(), 
			Box::new(|result: Result<_, _>| result.map(
				|(uuid,)| {
					Arc::new(Pipeline {
						uuid, 
						command_channel: 	command_channel,

						vertex_shader: 		info_copy.vertex_shader,
						fragment_shader: 	info_copy.fragment_shader,
					})
				}
			))
		);

		let _ = render_engine.command_channel.send(PipelineCommand::CreatePipeline { 
			vertex_shader: 		create_info.vertex_shader, 
			fragment_shader: 	create_info.fragment_shader, 

			surfaces:			create_info.surfaces.iter().map(|s| *s.get_uuid()).collect(),

			response:			response,
		}.into());

		return Box::new(future) as Box<dyn EngineFuture<_>>;
	}
}

impl Drop for Pipeline {
	fn drop(&mut self) {
		let _ = self.command_channel.send(PipelineCommand::DropPipeline { uuid: self.uuid }.into());
	}
}
