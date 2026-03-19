
use std::{
	collections::HashSet, 
	sync::{
		Arc, 
		mpsc::{SyncSender, sync_channel}
	}
};

use uuid::Uuid;
use vulkano::{
	format::Format, 
	pipeline::{
		GraphicsPipeline, 
		PipelineLayout, 
		PipelineShaderStageCreateInfo, 
		graphics::{
			GraphicsPipelineCreateInfo, 
			color_blend::ColorBlendState, 
			input_assembly::InputAssemblyState, 
			multisample::MultisampleState, 
			rasterization::RasterizationState, 
			subpass::PipelineRenderingCreateInfo, 
			vertex_input::{Vertex, VertexDefinition}, 
			viewport::{Viewport, ViewportState}
		}, 
		layout::PipelineDescriptorSetLayoutCreateInfo
	}
};

use crate::{
	mesh_data::Vertex3D, 
	render_engine::{EngineFuture, RenderEngine, RenderEngineCommand, RenderThread}, 
	shader::{Shader, ShaderType}
};

#[derive(Debug)]
pub struct Pipeline {
	uuid: Uuid,
	render_engine: Arc<RenderEngine>,

	shaders: Box<[Arc<Shader>]>,
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
			RenderEngineCommand::PipelineCommand(
				PipelineCommand::CreatePipeline { 
					sender: send, 
					shaders: shaders,
					engine: render_engine.clone(),
				}
			)
		).unwrap();

		return EngineFuture::new(recv);
	}
}

impl Drop for Pipeline {
	fn drop(&mut self) {
		self.render_engine.command_channel.send(
			RenderEngineCommand::PipelineCommand(
				PipelineCommand::DropPipeline { 
					uuid: self.uuid 
				}
			)
		).unwrap()
	}
}

#[derive(Debug)]
pub(crate) enum PipelineCommand {
	CreatePipeline {
		sender: SyncSender<Result<Arc<Pipeline>, ()>>,

		shaders: Box<[Arc<Shader>]>,
		engine: Arc<RenderEngine>,
	},
	DropPipeline {
		uuid: Uuid,
	}
}

impl RenderThread {
	pub(crate) fn process_pipeline_command(&mut self, command: PipelineCommand) {
		match command {
			PipelineCommand::CreatePipeline { sender, shaders , engine} => sender.send(self.create_pipeline(shaders, engine)).unwrap(),
			PipelineCommand::DropPipeline { uuid } => self.drop_pipeline(uuid),
		}
	}

	fn create_pipeline(&mut self, shaders: Box<[Arc<Shader>]>, engine: Arc<RenderEngine>) -> Result<Arc<Pipeline>, ()> {
		let uuid = Uuid::now_v7();

		let stages = shaders.iter()
			.map(|s| self.get_shader_internal(s.clone()).unwrap())
			.map(|s| s.entry_point.clone())
			.map(|e| PipelineShaderStageCreateInfo::new(e));

		let vertex_shader = self.get_shader_internal(
			shaders.iter()
				.find(|s| s.shader_type == ShaderType::Vertex).unwrap().clone()
		).unwrap();
		let vertex_input_state = Vertex3D::per_vertex().definition(&vertex_shader.entry_point).map_err(|_| ())?;

		let subpass = PipelineRenderingCreateInfo {
			color_attachment_formats: vec![Some(Format::R8G8B8A8_UNORM)],
			..Default::default()
		}.into();

		let layout = PipelineLayout::new(
			self.device.clone(),
			PipelineDescriptorSetLayoutCreateInfo::from_stages(
				shaders.iter()
					.map(|s| self.get_shader_internal(s.clone()).unwrap())
					.map(|s| s.entry_point.clone())
					.map(|e| PipelineShaderStageCreateInfo::new(e))
					.collect::<Vec<_>>().iter()
					.map(|i| i)
			).into_pipeline_layout_create_info(self.device.clone()).map_err(|_| ())?
		).map_err(|_| ())?;

		let pipeline = GraphicsPipeline::new(
			self.device.clone(), 
			None, 
			GraphicsPipelineCreateInfo {
				stages: stages.collect(),
				vertex_input_state: Some(vertex_input_state),
				input_assembly_state: Some(InputAssemblyState::default()),
				viewport_state: Some(ViewportState {
					viewports: vec![Viewport::default()].into(),
					..Default::default()
				}),
				rasterization_state: Some(RasterizationState::default()),
				multisample_state: Some(MultisampleState::default()),
				color_blend_state: Some(ColorBlendState {
					attachments: vec![Default::default()],
					..Default::default()
				}),
				subpass: Some(subpass),
				..GraphicsPipelineCreateInfo::layout(layout)
			}
		).map_err(|_| ())?;

		let internal = PipelineInternal {
			pipeline: pipeline,
			shaders: shaders.clone(),
		};

		self.pipelines.insert(uuid, internal);

		return Ok(Arc::new(Pipeline {
			uuid: uuid,
			render_engine: engine,
			shaders: shaders.clone(),
		}));
	}

	fn drop_pipeline(&mut self, uuid: Uuid) {
		self.pipelines.remove(&uuid);
	}
}

impl RenderThread {
	#[inline]
	pub(crate) fn get_pipeline_internal(&self, reference: Arc<Pipeline>) -> Option<&PipelineInternal> { self.pipelines.get(&reference.uuid) }
	#[inline]
	pub(crate) fn get_mut_pipeline_internal(&mut self, reference: Arc<Pipeline>) -> Option<&mut PipelineInternal> { self.pipelines.get_mut(&reference.uuid) }
}

pub(crate) struct PipelineInternal {
	pipeline: Arc<GraphicsPipeline>,
	shaders: Box<[Arc<Shader>]>,
}