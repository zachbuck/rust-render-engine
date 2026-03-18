use std::sync::{
	Arc, 
	mpsc::{Sender, channel}
};

use vulkano::buffer::{BufferContents, Subbuffer};

use crate::render_engine::{EngineFuture, RenderEngine, RenderEngineCommand, RenderThread};

pub struct MeshData {
	render_engine: Arc<RenderEngine>,
}

impl MeshData {
	pub fn new(render_engine: Arc<RenderEngine>) -> EngineFuture<Result<Arc<Self>, ()>> {
		let (send, recv) = channel();

		render_engine.command_channel.send(
			RenderEngineCommand::MeshDataCommand(
				MeshDataCommand::CreateMeshData { 
					sender: send
				}
			)
		).unwrap();

		return EngineFuture::new(recv);
	}
}

pub(crate) enum MeshDataCommand {
	CreateMeshData {
		sender: Sender<Result<Arc<MeshData>, ()>> 
	},
}

impl RenderThread {
	pub(crate) fn process_mesh_data_command(&mut self, command: MeshDataCommand) {
		match command {
			MeshDataCommand::CreateMeshData { sender } => sender.send(self.create_mesh_data()).unwrap(),
		}
	}

	fn create_mesh_data(&mut self) -> Result<Arc<MeshData>, ()> {
		todo!()
	}
}

struct MeshDataInternal {
	vertices: Vec<()>,
}

#[repr(C)]
#[derive(BufferContents)]
pub struct Vertex1 {
	pub x: [f32; 1]
}