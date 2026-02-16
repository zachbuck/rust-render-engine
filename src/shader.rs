
use std::{collections::HashSet, sync::Arc};

use foldhash::fast::RandomState;
use shaderc::{CompileOptions, ShaderKind};
use uuid::Uuid;
use vulkano::{
	pipeline::{
		DynamicState, 
		GraphicsPipeline, 
		PipelineLayout, 
		PipelineShaderStageCreateInfo, 
		graphics::{
			GraphicsPipelineCreateInfo, 
			color_blend::{ColorBlendAttachmentState, ColorBlendState}, 
			input_assembly::InputAssemblyState, 
			multisample::MultisampleState, 
			rasterization::RasterizationState, 
			subpass::PipelineRenderingCreateInfo, 
			vertex_input::{
				Vertex as _, 
				VertexDefinition
			}, 
			viewport::ViewportState
		}, 
		layout::PipelineDescriptorSetLayoutCreateInfo
	}, 
	shader::{EntryPoint, ShaderModule, ShaderModuleCreateInfo}
};

use crate::{RenderEngine, mesh_data::Vertex};

#[derive(Clone)]
pub struct Shader {
	uuid: Uuid,
}

pub(crate) struct ShaderInternal {
	entry_point: EntryPoint,
	shader_type: ShaderType,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ShaderType {
	Vertex,
	Fragment,
}

impl Into<ShaderKind> for ShaderType {
	fn into(self) -> ShaderKind {
		match self {
			ShaderType::Vertex => ShaderKind::Vertex,
			ShaderType::Fragment => ShaderKind::Fragment,
		}
	}
}

impl RenderEngine {
	pub fn create_shader(&mut self, source: String, shader_name: String, shader_type: ShaderType) -> Shader {
		let uuid = Uuid::now_v7();

		let mut options = CompileOptions::new().unwrap();
		options.add_macro_definition("EP", Some("main"));
		let binary_result = self.compiler.compile_into_spirv_assembly(
			&source, 
			shader_type.into(), 
			&shader_name, 
			"main", 
			Some(&options)
		).unwrap();

		let module = unsafe { ShaderModule::new(
			self.device.clone(),
			ShaderModuleCreateInfo::new(binary_result.as_binary())
		).unwrap() };

		let entry_point = module.entry_point("main").unwrap();

		let internal = ShaderInternal {
			entry_point: entry_point,
			shader_type: shader_type,
		};

		self.shaders.insert(uuid, internal);

		Shader {
			uuid: uuid
		}
	}
}

pub struct GraphicsProgram {
	pub(crate) uuid: Uuid,
}

#[derive(Clone)]
pub(crate) struct GraphicsProgramInternal {
	//pub(crate) shaders: Vec<Shader>,
	pub(crate) pipeline: Arc<GraphicsPipeline>,
}

impl RenderEngine {
	pub fn create_graphics_program(&mut self, shaders: Vec<Shader>) -> GraphicsProgram {
		let uuid = Uuid::now_v7();

		let internal = shaders.iter()
			.map(|s| self.shaders.get(&s.uuid).unwrap());

		let stages = internal.clone()
			.map(|s| PipelineShaderStageCreateInfo::new(s.entry_point.clone()));

		let vertex_shader = internal.clone()
			.find(|s| s.shader_type == ShaderType::Vertex).unwrap();

		let vertex_input_state = Vertex::per_vertex()
			.definition(&vertex_shader.entry_point).unwrap();

		let mut dynamic_state = HashSet::with_hasher(RandomState::default());
		dynamic_state.insert(DynamicState::ViewportWithCount);

		let subpass = PipelineRenderingCreateInfo {
			color_attachment_formats: vec![Some(vulkano::format::Format::R8G8B8A8_UNORM)],
			..Default::default()
		};

		let layout= PipelineLayout::new(
			self.device.clone(),
			PipelineDescriptorSetLayoutCreateInfo::from_stages(stages.clone().collect::<Vec<_>>().iter().map(|s| s))
				.into_pipeline_layout_create_info(self.device.clone()).unwrap()
		).unwrap();

		let pipeline = GraphicsPipeline::new(
			self.device.clone(),
			None,
			GraphicsPipelineCreateInfo {
				stages: stages.collect(),
				vertex_input_state: Some(vertex_input_state),
				input_assembly_state: Some(InputAssemblyState::default()),
				viewport_state: Some(ViewportState::default()),
				rasterization_state: Some(RasterizationState::default()),
				multisample_state: Some(MultisampleState::default()),
				color_blend_state: Some(ColorBlendState {
					attachments: vec![ColorBlendAttachmentState::default()],
					..Default::default()
				}),
				dynamic_state: dynamic_state,
				subpass: Some(subpass.into()),
				..GraphicsPipelineCreateInfo::layout(layout)
			}
		).unwrap();

		let internal = GraphicsProgramInternal {
			//shaders: shaders,
			pipeline: pipeline,
		};

		self.graphics_programs.insert(uuid, internal);

		GraphicsProgram {
			uuid: uuid,
		}
	}
}