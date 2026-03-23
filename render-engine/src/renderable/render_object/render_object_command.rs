
use std::sync::{
	Arc, 
	mpsc::SyncSender
};

use uuid::Uuid;

use crate::{
	mesh_data::MeshData, 
	pipeline::Pipeline, 
	render_engine::{
		RenderEngine, 
		render_command::RenderEngineCommand, 
		render_thread::RenderThread
	}, 
	renderable::render_object::{
		RenderObject, 
		render_object_internal::RenderObjectInternal
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
		}
	}

	fn create_render_object(&mut self, mesh_data: Arc<MeshData>, pipeline: Arc<Pipeline>, render_engine: Arc<RenderEngine>) -> Result<Arc<RenderObject>, ()> {
		let uuid = Uuid::now_v7();

		let internal = Box::new(RenderObjectInternal {
			mesh: mesh_data.clone(),
			pipeline: pipeline.clone(),
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
}
