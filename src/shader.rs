
use std::sync::Arc;

use shaderc::{CompileOptions, ShaderKind};
use uuid::Uuid;
use vulkano::{
	descriptor_set::layout::DescriptorSetLayout, pipeline::{GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo, graphics::GraphicsPipelineCreateInfo, layout::{PipelineDescriptorSetLayoutCreateInfo, PipelineLayoutCreateInfo}}, shader::{EntryPoint, ShaderModule, ShaderModuleCreateInfo}
};

use crate::RenderEngine;

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
	pub(crate) shaders: Vec<Shader>,
	pub(crate) pipeline: Arc<GraphicsPipeline>,
}

impl RenderEngine {
	pub fn create_graphics_program(&mut self, shaders: Vec<Shader>) -> GraphicsProgram {
		let uuid = Uuid::now_v7();

		let internal_shaders = shaders.iter()
			.map(|s| self.shaders.get(&s.uuid).unwrap());

		let vertex_shader = internal_shaders.clone()
			.find(|s| s.shader_type == ShaderType::Vertex)
			.unwrap();

		let stages = internal_shaders
			.map(|s| &PipelineShaderStageCreateInfo::new(s.entry_point.clone()));

		let layout = PipelineLayout::new(
			self.device.clone(),
			PipelineDescriptorSetLayoutCreateInfo::from_stages(stages.clone())
				.into_pipeline_layout_create_info(self.device.clone())
				.unwrap()
		).unwrap();

		let pipeline = GraphicsPipeline::new(
			self.device.clone(),
			None,
			GraphicsPipelineCreateInfo {
				stages: stages.map(|s| s.clone()).collect(),
				vertex_input_state: todo!(),
				input_assembly_state: todo!(),
				viewport_state: todo!(),
				rasterization_state: todo!(),
				multisample_state: todo!(),
				color_blend_state: todo!(),
				dynamic_state: todo!(),
				subpass: todo!(),
				..GraphicsPipelineCreateInfo::layout(layout)
			}
		).unwrap();

		let internal = GraphicsProgramInternal {
			shaders: todo!(),
			pipeline: todo!(),
		};

		self.graphics_programs.insert(uuid, internal);

		GraphicsProgram {
			uuid: uuid,
		}
	}
}