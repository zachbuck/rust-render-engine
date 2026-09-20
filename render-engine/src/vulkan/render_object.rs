
use uuid::Uuid;

use crate::{
	engine_command::RenderObjectCommand, 
	vulkan::render_thread::RenderThread,
};

pub struct RenderObject {
	pub mesh_data: Uuid,
	pub pipeline: Uuid,
}

impl RenderThread {
	pub fn process_render_object_command(&mut self, command: Box<RenderObjectCommand>) {
		match *command {
			RenderObjectCommand::CreateRenderObject { mesh_data, pipeline, response } => response.send(self.create_render_object(mesh_data, pipeline)),
			RenderObjectCommand::DropRenderObject { uuid } => self.drop_render_object(uuid),
		}
	}

	fn create_render_object(&mut self, mesh_data: Uuid, pipeline: Uuid) -> Result<(Uuid,), ()> {
		let uuid = Uuid::now_v7();

		let render_object = RenderObject {
			mesh_data: mesh_data,
			pipeline: pipeline,
		};

		self.render_objects.insert(uuid, render_object);

		Ok((uuid,))
	}

	fn drop_render_object(&mut self, uuid: Uuid) {
		self.render_objects.remove(&uuid);
	}
}
