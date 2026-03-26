
use std::sync::{
	Arc, 
	mpsc::SyncSender,
};

use uuid::Uuid;

use crate::{
	mesh_data::MeshData, 
	pipeline::Pipeline, 
	render_engine::{
		RenderEngine, 
		render_command::RenderEngineCommand, 
		render_resources::RenderResources, 
		render_thread::RenderThread,
	}, 
	renderable::{
		descriptor_set_data::{DescriptorData, DescriptorSetData}, 
		render_object::{
			RenderObject, 
			render_object_internal::RenderObjectInternal,
		},
	},
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

		let internal_pipeline = Self::get_pipeline(&self.pipelines, &pipeline.uuid).unwrap();

		let set_count = pipeline.descriptor_requirements.sets.len();
		let mut descriptor_data = Vec::with_capacity(set_count);
		for i in 0..set_count {
			let requirements = &pipeline.descriptor_requirements.sets[i];
			let layout = &internal_pipeline.descriptor_layouts[i];

			descriptor_data.push(DescriptorSetData::new(requirements, &self.descriptor_allocator, layout, &self.default_resources)?);
		}
		let descriptor_data = descriptor_data.into_boxed_slice();

		let internal = Box::new(RenderObjectInternal {
			mesh: mesh_data.clone(),
			pipeline: pipeline.clone(),

			descriptor_data: descriptor_data,
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
		let render_resources = RenderResources::new(&self.mesh_data, &self.pipelines, &self.textures);

		let render_object_internal = Self::get_mut_render_object(&mut self.renderables, &uuid).unwrap();

		let descriptor_set = render_object_internal.descriptor_data.iter_mut().find(|d| d.set == set).unwrap();
		descriptor_set.update_binding(binding, data, &render_resources)?;

		Ok(())
	}
}
