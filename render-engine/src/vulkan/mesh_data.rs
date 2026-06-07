
use uuid::Uuid;
use vulkano::buffer::Subbuffer;

use crate::{
	interface::{
		data_format::{
			index::IndexCollection,
			vertex::{VertexCollection, Vertex2D, Vertex3D},
		},
		engine_command::MeshDataCommand,
	}, 
	vulkan::render_engine::RenderEngine,
};

pub struct MeshData {
	// vertices: VertexBuffer,
	// indices: IndexBuffer,
}

// enum VertexBuffer {
// 	Vertex2D(Subbuffer<[Vertex2D]>),
// 	Vertex3D(Subbuffer<[Vertex3D]>),
// }

// enum IndexBuffer {
// 	U8(Subbuffer<[u8]>),
// 	U16(Subbuffer<[u16]>),
// 	U32(Subbuffer<[u32]>),
// }

impl RenderEngine {
	pub fn process_mesh_data_command(&mut self, command: Box<MeshDataCommand>) {
		match *command {
			MeshDataCommand::CreateMeshData { vertices, indices, response } => response.send(self.create_mesh_data(vertices, indices)),
			MeshDataCommand::DropMeshData { uuid } => self.drop_mesh_data(uuid),
		}
	}

	fn create_mesh_data(&mut self, vertices: VertexCollection, indices: IndexCollection) -> Result<(Uuid,), ()> {
		println!("MeshData::new");

		let uuid = Uuid::now_v7();

		let mesh_data = MeshData {};
		self.mesh_data.insert(uuid, mesh_data);

		Ok((uuid,))
	}

	fn drop_mesh_data(&mut self, uuid: Uuid) -> () {
		println!("MeshData::drop");

		self.mesh_data.remove(&uuid);
	}
}
