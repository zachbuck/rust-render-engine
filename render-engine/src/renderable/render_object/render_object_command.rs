
use std::{
	collections::HashMap, 
	sync::{
		Arc, 
		mpsc::SyncSender,
	},
};

use uuid::Uuid;
use vulkano::descriptor_set::DescriptorSet;

use crate::{
	macros::error_map, mesh_data::MeshData, pipeline::Pipeline, render_engine::{
		RenderEngine, render_command::RenderEngineCommand, render_resources::RenderResources, render_thread::RenderThread
	}, renderable::{
		descriptor_set_data::{DescriptorData, DescriptorDataInternal, DescriptorDataType}, 
		render_object::{
			RenderObject, 
			render_object_internal::RenderObjectInternal,
		},
	}
};

#[derive(Debug)]
pub(crate) enum RenderObjectCommand {
	CreateRenderObject {
		sender: SyncSender<Result<Arc<RenderObject>, ()>>,

		mesh_data: Arc<MeshData>,
		pipeline: Arc<Pipeline>,
		render_engine: Arc<RenderEngine>,
	},
	DropRenderObject {
		uuid: Uuid
	},

	UpdateDescriptor {
		sender: SyncSender<Result<(), ()>>,

		uuid: Uuid,
		set: u32,
		binding: u32,

		data: DescriptorData,
	}
}

impl Into<RenderEngineCommand> for RenderObjectCommand {
	fn into(self) -> RenderEngineCommand {
		RenderEngineCommand::RenderObjectCommand(self)
	}
}

impl RenderThread {
	pub(crate) fn process_render_object_command(&mut self, command: RenderObjectCommand) {
		match command {
			RenderObjectCommand::CreateRenderObject { sender, mesh_data, pipeline, render_engine } => {let _ = sender.send(self.create_render_object(mesh_data, pipeline, render_engine));},
			RenderObjectCommand::DropRenderObject { uuid } => self.drop_render_object(uuid),

			RenderObjectCommand::UpdateDescriptor { sender, uuid, set, binding, data } => {let _ = sender.send(self.update_descriptor(uuid, set, binding, data));},
		}
	}

	fn create_render_object(&mut self, mesh_data: Arc<MeshData>, pipeline: Arc<Pipeline>, render_engine: Arc<RenderEngine>) -> Result<Arc<RenderObject>, ()> {
		let uuid = Uuid::now_v7();

		let pipeline_internal = Self::get_pipeline(&self.pipelines, &pipeline.uuid).unwrap();

		let descriptor_requirmenets = &pipeline.descriptor_requirements;
		let mut descriptors = HashMap::new();
		for ((set, binding), descriptor_type, _) in &descriptor_requirmenets.descriptors {
			descriptors
				.entry(set)
				.or_insert(Vec::new())
				.push((binding, descriptor_type));
		}

		let mut descriptor_data = Vec::new();
		for (set, bindings) in descriptors {
			let mut descriptor_default_data = Vec::with_capacity(bindings.len());
			for (binding, descriptor_type) in bindings {
				descriptor_default_data.push((*binding, DescriptorDataInternal::get_default(descriptor_type, &self.default_resources, &self.buffer_allocator)));
			}
			let descriptor_default_data = descriptor_default_data.into_boxed_slice();

			let descriptor_set = DescriptorSet::new(
				self.descriptor_allocator.clone(), 
				pipeline_internal.descriptor_layouts.get(set).unwrap().clone(), 
				descriptor_default_data.iter().map(|x| DescriptorDataInternal::get_descriptor_write(x)), 
				[],
			).map_err(error_map!())?;

			descriptor_data.push((*set, descriptor_set, descriptor_default_data));
		}
		descriptor_data.sort_by_key(|(set, _, _)| *set);

		let internal = Box::new(RenderObjectInternal {
			mesh: mesh_data.clone(),
			pipeline: pipeline.clone(),

			descriptor_data: descriptor_data.into_boxed_slice(),
		});

		self.renderables.insert(uuid, internal);

		Ok(Arc::new(RenderObject { 
			uuid: uuid, 
			render_engine: render_engine.clone(), 

			mesh: mesh_data.clone(), 
			pipeline: pipeline.clone(),
		}))
	}

	fn drop_render_object(&mut self, uuid: Uuid) {
		self.renderables.remove(&uuid);
	}

	fn update_descriptor(&mut self, uuid: Uuid, set: u32, binding: u32, data: DescriptorData) -> Result<(), ()> {
		let render_data = RenderResources::new(&self.mesh_data, &self.pipelines, &self.textures);

		let render_object_internal = Self::get_mut_render_object(&mut self.renderables, &uuid).unwrap();

		let (_, descriptor_set, bindings) = render_object_internal.descriptor_data.iter_mut().find(|(s, _, _)| *s == set).unwrap();
		let (_, descriptor_data) = bindings.iter_mut().find(|(b, _)| *b == binding).unwrap();
		
		match data {
			DescriptorData::CombinedImageSampler(texture) => {
				let data_type = &mut descriptor_data.descriptor_data;
				if let DescriptorDataType::CombinedImageSampler(_, _) = data_type {
					let texture = render_data.get_texture(&texture).unwrap();

					*data_type = DescriptorDataType::CombinedImageSampler(texture.image.clone(), texture.sampler.clone());
				} else {
					return Err(())
				}

				unsafe {
					descriptor_set.update_by_ref(
						bindings.iter().map(|b| DescriptorDataInternal::get_descriptor_write(b)), 
						[],
					).map_err(error_map!())?;
				}
			},
			DescriptorData::UniformBuffer(data) => {
				let data_type = &mut descriptor_data.descriptor_data;

				if let DescriptorDataType::UniformBuffer(buf) = data_type {
					let mut write_guard = buf.write().unwrap();
					for i in 0..data.len() {
						(*write_guard)[i] = data[i];
					}
				} else {
					return Err(())
				}
			},
		}

		Ok(())
	}
}
