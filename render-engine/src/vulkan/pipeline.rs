
use std::{
	collections::{BTreeMap, HashMap, HashSet}, 
	sync::Arc,
};

use foldhash::fast::RandomState;
use parsing::spir_v::{
	data_type::DataType, 
	shader::DescriptorCollection,
};
use uuid::Uuid;
use vulkano::{
	descriptor_set::layout::{DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutCreateInfo, DescriptorType}, 
	device::Device, 
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
			vertex_input::{Vertex, VertexDefinition}, 
			viewport::ViewportState,
		}, 
		layout::PipelineLayoutCreateInfo,
	}, 
	render_pass::RenderPass, 
	shader::ShaderStages,
};

use crate::{
	data_formats::Vertex3D, 
	engine_command::PipelineCommand, 
	macros::error_to_unit_type, 
	shader::Shader, 
	vulkan::{
		render_thread::RenderThread, 
		shader::ShaderModule,
	},
};

pub struct ShaderCollection {
	vertex_shader: 		Arc<Shader>,
	fragment_shader: 	Arc<Shader>,

	descriptors:		DescriptorCollection,
}

pub struct Pipeline {
	pub pipeline: Arc<GraphicsPipeline>,
}

impl RenderThread {
	pub fn process_pipeline_command(&mut self, command: Box<PipelineCommand>) -> () {
		match *command {
			PipelineCommand::CreatePipeline { vertex_shader, fragment_shader, surfaces, response } => {
				let result = self.create_shader_collection(vertex_shader, fragment_shader);
				response.send(result);

				if !result.is_err() {
					let (shader_collection,) = result.unwrap();
					for surface in surfaces {
						let render_pass = self.surfaces.get(&surface).unwrap().get_renderpass();
						self.ensure_pipeline(shader_collection, *render_pass);
					}
				}
			},
			PipelineCommand::DropPipeline { uuid } => self.drop_pipeline(uuid),
		}
	}

	fn create_shader_collection(&mut self, vertex_shader: Arc<Shader>, fragment_shader: Arc<Shader>) -> Result<(Uuid,), ()> {
		let uuid = Uuid::now_v7();

		let mut descriptors = vertex_shader.get_uniforms().clone();
		descriptors = descriptors.merge_with(fragment_shader.get_uniforms().clone())?;

		let shader_collection = ShaderCollection {
			vertex_shader: 		vertex_shader,
			fragment_shader: 	fragment_shader,

			descriptors:		descriptors,
		};

		self.pipelines.insert(uuid, shader_collection);

		return Ok((uuid,));
	}

	fn ensure_pipeline(&mut self, shader_collection: Uuid, render_pass: Uuid) {
		self.linked_pipelines
			.entry(render_pass)
			.or_insert(HashMap::new())
			.entry(shader_collection)
			.or_insert(Self::link_pipeline(
				self.pipelines.get(&shader_collection).unwrap(), 
				self.render_passes.get(&render_pass).unwrap().clone(), 
				&self.shader_modules,
				self.device.clone(),
			));
	}

	fn link_pipeline(shader_collection: &ShaderCollection, render_pass: Arc<RenderPass>, shader_list: &HashMap<Uuid, ShaderModule>, device: Arc<Device>) -> Pipeline {
		let mut internal_shader_collection = Vec::new();
		let vertex_shader_internal = shader_list.get(shader_collection.vertex_shader.get_uuid()).unwrap();
		internal_shader_collection.push(vertex_shader_internal);
		internal_shader_collection.push(shader_list.get(shader_collection.fragment_shader.get_uuid()).unwrap());

		let stages = internal_shader_collection.iter().map(|s| PipelineShaderStageCreateInfo::new(s.entry_point.clone()));

		let vertex_input_state = Vertex3D::per_vertex().definition(&vertex_shader_internal.entry_point).unwrap();

		const DYNAMIC_STATE: [DynamicState; 1] = [DynamicState::Viewport];
		let mut dynamic_state = HashSet::with_capacity_and_hasher(DYNAMIC_STATE.len(), RandomState::default());
		for ds in DYNAMIC_STATE {
			dynamic_state.insert(ds);
		}

		let set_layouts = shader_collection.descriptors.descriptors.iter()
			.map(|ds| {
				let mut bindings = BTreeMap::new();
				ds.bindings.iter()
					.for_each(|db| {
						let descriptor_type = match &db.data_type {
								DataType::Image { dimension: _, pixel_format: _, texture_format: _ } => DescriptorType::StorageImage,
								DataType::Sampler => DescriptorType::Sampler,
								DataType::ImageSampler { dimension: _, pixel_format: _, texture_format: _ } => DescriptorType::CombinedImageSampler,
								_ => DescriptorType::StorageBuffer,
							};

						bindings.insert(db.binding, DescriptorSetLayoutBinding {
							stages: ShaderStages::all_graphics(),
							..DescriptorSetLayoutBinding::descriptor_type(descriptor_type)
						});
					});

				DescriptorSetLayout::new(
					device.clone(), 
					DescriptorSetLayoutCreateInfo {
						bindings: bindings,
						..Default::default()
					},
				).unwrap()
			}).collect::<Vec<_>>();

		let layout = PipelineLayout::new(
			device.clone(), 
			PipelineLayoutCreateInfo {
				set_layouts,
				..Default::default()
			}
		).unwrap();

		let internal = GraphicsPipeline::new(
			device.clone(), 
			None, 
			GraphicsPipelineCreateInfo {
				stages: 				stages.collect(),
				vertex_input_state: 	Some(vertex_input_state),
				input_assembly_state: 	Some(InputAssemblyState::default()),
				tessellation_state: 	None,
				viewport_state: 		Some(ViewportState::default()),
				rasterization_state: 	Some(RasterizationState::default()),
				multisample_state: 		Some(MultisampleState::default()),
				depth_stencil_state:	None,
				color_blend_state:		Some(ColorBlendState {
					attachments: vec![ColorBlendAttachmentState::default()],
					..Default::default()
				}),
				dynamic_state:			dynamic_state,
				subpass:				Some(render_pass.first_subpass().into()),
				..GraphicsPipelineCreateInfo::layout(layout)
			}
		).map_err(error_to_unit_type!()).unwrap();

		Pipeline {
			pipeline: internal,
		}
	}

	fn drop_pipeline(&mut self, uuid: Uuid) -> () {
		self.pipelines.remove(&uuid);

		for (_render_pass, pipelines) in &mut self.linked_pipelines {
			pipelines.remove(&uuid);
		}
	}
}
