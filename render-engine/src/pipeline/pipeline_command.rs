
use std::{
	collections::HashSet, 
	sync::{
		Arc, 
		mpsc::SyncSender
	}
};

use foldhash::fast::RandomState;
use uuid::Uuid;
use vulkano::{
	format::Format, 
	pipeline::{
		DynamicState, 
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
			viewport::ViewportState
		}, 
		layout::{PipelineLayoutCreateFlags, PipelineLayoutCreateInfo}
	}
};

use crate::{
	macros::error_map, 
	mesh_data::Vertex3D, 
	pipeline::{
		Pipeline, 
		pipeline_internal::PipelineInternal
	}, 
	render_engine::{
		RenderEngine, 
		render_command::RenderEngineCommand, 
		render_thread::RenderThread
	}, 
	shader::{
		Shader, 
		ShaderType, 
		descriptor_requirements::DescriptorRequirements
	},
};

#[derive(Debug)]
pub(crate) enum PipelineCommand {
	CreatePipeline {
		sender: SyncSender<Result<Arc<Pipeline>, ()>>,

		shaders: Box<[Arc<Shader>]>,
		engine: Arc<RenderEngine>,
	},
	GetPipelines {
		sender: SyncSender<Result<Box<[Arc<Pipeline>]>, ()>>,
	},
	DropPipeline {
		uuid: Uuid,
	}
}

impl Into<RenderEngineCommand> for PipelineCommand {
	fn into(self) -> RenderEngineCommand {
		RenderEngineCommand::PipelineCommand(self)
	}
}

impl RenderThread {
	pub(crate) fn process_pipeline_command(&mut self, command: PipelineCommand) {
		match command {
			PipelineCommand::CreatePipeline { sender, shaders , engine} => { let _ = sender.send(self.create_pipeline(shaders, engine)); },
			PipelineCommand::GetPipelines { sender } => { let _ = sender.send(self.get_pipelines()); },
			PipelineCommand::DropPipeline { uuid } => self.drop_pipeline(uuid),
		}
	}

	fn create_pipeline(&mut self, shaders: Box<[Arc<Shader>]>, engine: Arc<RenderEngine>) -> Result<Arc<Pipeline>, ()> {
		let uuid = Uuid::now_v7();

		let stages = shaders.iter()
			.map(|s| Self::get_shader_internal(&self.shaders, &s.uuid).unwrap())
			.map(|s| s.entry_point.clone())
			.map(|e| PipelineShaderStageCreateInfo::new(e));

		let vertex_shader = Self::get_shader_internal(
			&self.shaders,
			&shaders.iter()
				.find(|s| s.shader_type == ShaderType::Vertex).unwrap()
				.uuid
		).unwrap();
		let vertex_input_state = Vertex3D::per_vertex().definition(&vertex_shader.entry_point).map_err(error_map!())?;

		let subpass = PipelineRenderingCreateInfo {
			color_attachment_formats: vec![Some(Format::R8G8B8A8_UNORM)],
			..Default::default()
		}.into();

		let descriptor_requirements = DescriptorRequirements::combine(&shaders.iter().map(|s| s.descriptor_requirements.clone()).collect::<Vec<_>>());

		let descriptor_set_layouts = descriptor_requirements.get_descriptor_layouts(&self.device)?;
		let layout = PipelineLayout::new(
			self.device.clone(),
			PipelineLayoutCreateInfo {
				flags: PipelineLayoutCreateFlags::empty(),
				set_layouts: descriptor_set_layouts.values().map(|l| l.clone()).collect(),
				push_constant_ranges: Vec::new(),
				..Default::default()
			}
		).map_err(error_map!())?;

		let mut dynamic_state = HashSet::with_hasher(RandomState::default());
		dynamic_state.insert(DynamicState::ViewportWithCount);
		dynamic_state.insert(DynamicState::ScissorWithCount);

		let pipeline = GraphicsPipeline::new(
			self.device.clone(), 
			None, 
			GraphicsPipelineCreateInfo {
				stages: stages.collect(),
				vertex_input_state: Some(vertex_input_state),
				input_assembly_state: Some(InputAssemblyState::default()),
				viewport_state: Some(ViewportState {
					viewports: vec![].into(),
					scissors: vec![].into(),
					..Default::default()
				}),
				rasterization_state: Some(RasterizationState::default()),
				multisample_state: Some(MultisampleState::default()),
				color_blend_state: Some(ColorBlendState {
					attachments: vec![Default::default()],
					..Default::default()
				}),
				dynamic_state: dynamic_state,
				subpass: Some(subpass),
				..GraphicsPipelineCreateInfo::layout(layout)
			}
		).map_err(error_map!())?;

		let reference = Arc::new(Pipeline {
			uuid: uuid,
			render_engine: engine,
			shaders: shaders.clone(),
			descriptor_requirements: descriptor_requirements,
		});

		let internal = PipelineInternal {
			reference: Arc::downgrade(&reference),
			pipeline: pipeline,
			descriptor_layouts: descriptor_set_layouts,
		};

		self.pipelines.insert(uuid, internal);

		return Ok(reference)
	}

	fn get_pipelines(&mut self) -> Result<Box<[Arc<Pipeline>]>, ()> {
		Ok(self.pipelines.values().filter_map(|p| p.reference.upgrade()).collect())
	}

	fn drop_pipeline(&mut self, uuid: Uuid) {
		self.pipelines.remove(&uuid);
	}
}
