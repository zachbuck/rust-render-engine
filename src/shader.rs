
use std::{collections::{BTreeMap, HashSet}, sync::Arc};

use foldhash::fast::RandomState;
use shaderc::{CompileOptions, ShaderKind};
use uuid::Uuid;
use vulkano::{
	descriptor_set::layout::{DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutCreateFlags, DescriptorSetLayoutCreateInfo}, pipeline::{
		DynamicState, GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo, graphics::{
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
		}, layout::{PipelineDescriptorSetLayoutCreateInfo, PipelineLayoutCreateInfo}
	}, shader::{DescriptorBindingRequirements, EntryPoint, ShaderModule, ShaderModuleCreateInfo, ShaderStages, spirv::ExecutionModel}
};

use crate::{RenderEngine, mesh_data::Vertex, unwrap_option_or_none, unwrap_result_or_none};

#[derive(Clone)]
pub struct Shader {
	uuid: Uuid,
}

pub(crate) struct ShaderInternal {
	entry_point: EntryPoint,
	shader_type: ShaderType,

	descriptors: Vec<Descriptor>,
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

impl From<ExecutionModel> for ShaderType {
	fn from(value: ExecutionModel) -> Self {
		match value {
			ExecutionModel::Vertex => ShaderType::Vertex,
			ExecutionModel::Fragment => ShaderType::Fragment,
			_ => todo!()
		}
	}
}

impl Into<ShaderStages> for ShaderType {
	fn into(self) -> ShaderStages {
		match self {
			ShaderType::Vertex => ShaderStages::VERTEX,
			ShaderType::Fragment => ShaderStages::FRAGMENT,
		}
	}
}

#[derive(Clone)]
#[derive(Debug)]
pub struct Descriptor {
	set: u32,
	binding: u32,

	descriptor_type: DescriptorType,
}

impl Descriptor {
	pub fn uniform_buffer(set: u32, binding: u32, uniforms: &[UniformType]) -> Self {
		return Descriptor {
			set: set,
			binding: binding,
			descriptor_type: DescriptorType::UniformBuffer(uniforms.to_vec()),
		};
	}

	fn is_compatable_with_requirements(&self, requirements: &DescriptorBindingRequirements) -> bool {
		match self.descriptor_type {
			DescriptorType::UniformBuffer(_) => requirements.descriptor_types.contains(&vulkano::descriptor_set::layout::DescriptorType::UniformBuffer),
			DescriptorType::Unknown => true,
		}
	}

	fn is_compatable_with_descriptor(&self, other: &Descriptor) -> bool {
		if self.descriptor_type != other.descriptor_type { return false; }
		return true;
	}

	fn from_requirements(set: u32, binding: u32, requirements: &DescriptorBindingRequirements) -> Self {
		if requirements.descriptor_types.contains(&vulkano::descriptor_set::layout::DescriptorType::UniformBuffer) {
			return Descriptor {
				set: set,
				binding: binding,
				descriptor_type: DescriptorType::UniformBuffer(Vec::new()),
			};
		} else {
			return Descriptor {
				set: set,
				binding: binding,
				descriptor_type: DescriptorType::Unknown,
			}
		}
	}
}

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq, Eq)]
enum DescriptorType {
	UniformBuffer(Vec<UniformType>),
	Unknown,
}

impl Into<vulkano::descriptor_set::layout::DescriptorType> for DescriptorType {
	fn into(self) -> vulkano::descriptor_set::layout::DescriptorType {
		match self {
			DescriptorType::UniformBuffer(_) => vulkano::descriptor_set::layout::DescriptorType::UniformBuffer,
			DescriptorType::Unknown => unimplemented!(),
		}
	}
}

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq, Eq)]
pub enum UniformType {
	Mat4
}

impl RenderEngine {
	pub fn compile_shader(&self, shader_source: String, shader_name: String, shader_type: ShaderType) -> Result<(Vec<u32>, Option<String>), String> {
		let mut options = unwrap_result_or_none!(CompileOptions::new(), "".to_string());
		options.add_macro_definition("EP", Some("main"));

		let result = self.compiler.compile_into_spirv(
			&shader_source, 
			shader_type.into(), 
			&shader_name, 
			"main", 
			Some(&options)
		);

		if let Err(e) = result { return Err(e.to_string()); }

		let result = result.unwrap();

		let shader_binary = result.as_binary().to_vec();
		let warnings;
		if result.get_num_warnings() != 0 {
			warnings = Some(result.get_warning_messages())
		} else {
			warnings = None;
		}

		return Ok((shader_binary, warnings))
	}

	pub fn create_shader(&mut self, shader_binary: Vec<u32>, descriptor_info: &[Descriptor]) -> Result<Shader, ()> {
		let uuid = Uuid::now_v7();

		let module = unsafe { unwrap_result_or_none!(ShaderModule::new(
				self.device.clone(),
				ShaderModuleCreateInfo::new(shader_binary.as_slice())
			))
		};

		let entry_point = unwrap_option_or_none!(module.entry_point("main"));

		let shader_type = entry_point.info().execution_model.into();

		let mut descriptors = Vec::new();

		for ((set, binding), requirements) in &entry_point.info().descriptor_binding_requirements {
			let descriptor = descriptor_info.iter().find(|d| d.set == *set && d.binding == *binding);
			if descriptor.is_some_and(|d| d.is_compatable_with_requirements(requirements)) {
				descriptors.push(descriptor.unwrap().clone());
			} else {
				descriptors.push(Descriptor::from_requirements(*set, *binding, requirements))
			}
		}

		descriptors.sort_by(|a, b| {
			if a.set != b.set {
				return a.set.cmp(&b.set);
			} else {
				return a.binding.cmp(&b.binding);
			}
		});

		let internal = ShaderInternal {
			entry_point: entry_point,
			shader_type: shader_type,
			descriptors: descriptors,
		};

		self.shaders.insert(uuid, internal);

		Ok(Shader { uuid: uuid })
	}
}

pub struct GraphicsProgram {
	pub(crate) uuid: Uuid,
}

#[derive(Clone)]
pub(crate) struct GraphicsProgramInternal {
	//pub(crate) shaders: Vec<Shader>,
	pub(crate) pipeline: Arc<GraphicsPipeline>,
	descriptors: Vec<Descriptor>,
}

impl RenderEngine {
	pub fn create_graphics_program(&mut self, shaders: Vec<Shader>) -> Result<GraphicsProgram, ()> {
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

		let mut program_descriptors: Vec<(ShaderStages, Descriptor)> = Vec::new();
		for shader in internal {
			for descriptor in &shader.descriptors {
				let compare = program_descriptors.iter_mut().find(|(_, d)| d.set == descriptor.set && d.binding == descriptor.binding);
				if compare.is_none() {
					program_descriptors.push((shader.shader_type.into(), descriptor.clone()))
				} else {
					let compare = compare.unwrap();
					if !compare.1.is_compatable_with_descriptor(descriptor) {
						return Err(());
					} else {
						compare.0 = compare.0.union(shader.shader_type.into());
					}
				}
			}
		}

		println!("{:?}", program_descriptors);

		let descriptor_sets = program_descriptors.chunk_by(|(_, a), (_, b)| a.set == b.set);
		let mut set_layouts = Vec::new();
		for set in descriptor_sets {
			let bindings = BTreeMap::new();

			for (stages, descriptor) in set {
				DescriptorSetLayoutBinding {
					..DescriptorSetLayoutBinding::descriptor_type(descriptor.descriptor_type.into())
				}
			}
			
			let set_layout = DescriptorSetLayout::new(
				self.device.clone(),
				DescriptorSetLayoutCreateInfo {
					flags: DescriptorSetLayoutCreateFlags::PUSH_DESCRIPTOR,
					bindings: bindings,
					..Default::default()
				}
			).unwrap();
		}

		let layout = PipelineLayout::new(
			self.device.clone(),
			PipelineLayoutCreateInfo {
				set_layouts: set_layouts,
				..Default::default()
			}
		).unwrap();

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
				viewport_state: Some(ViewportState {
					viewports: vec![].into(),
					..Default::default()
				}),
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
			descriptors: program_descriptors.iter().map(|(_, d)| d.clone()).collect(),
		};

		self.graphics_programs.insert(uuid, internal);

		Ok(GraphicsProgram {
			uuid: uuid,
		})
	}
}