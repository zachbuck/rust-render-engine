
use std::sync::{
	Arc, 
	mpsc::SyncSender,
};

use uuid::Uuid;
use vulkano::{
	buffer::{Buffer, BufferCreateInfo, BufferUsage}, 
	memory::allocator::{AllocationCreateInfo, MemoryTypeFilter}
};

use crate::{
	mesh_data::{
		MeshData, 
		Vertex3D, 
		mesh_data_internal::MeshDataInternal
	}, 
	render_engine::{
		RenderEngine, 
		render_thread::RenderThread
	}
};

#[derive(Debug)]
pub(crate) enum MeshDataCommand {
	CreateMeshData {
		sender: 	SyncSender<Result<Arc<MeshData>, ()>>,

		vertices:	Box<[Vertex3D]>,
		indices:	Box<[u16]>,
		engine: 	Arc<RenderEngine>,
	},
	DropMeshData {
		uuid:		Uuid,
	}
}

impl RenderThread {
	pub(crate) fn process_mesh_data_command(&mut self, command: MeshDataCommand) {
		match command {
			MeshDataCommand::CreateMeshData { sender, vertices, indices, engine } => sender.send(self.create_mesh_data(&vertices, &indices, engine)).unwrap(),
			MeshDataCommand::DropMeshData { uuid } => self.drop_mesh_data(uuid),
		}
	}

	fn create_mesh_data(&mut self, vertices: &[Vertex3D], indices: &[u16], engine: Arc<RenderEngine>) -> Result<Arc<MeshData>, ()> {
		let uuid = Uuid::now_v7();

		let vertices = Buffer::from_iter(
			self.buffer_allocator.clone(),
			BufferCreateInfo {
				usage: BufferUsage::VERTEX_BUFFER,
				..Default::default()
			},
			AllocationCreateInfo {
				memory_type_filter: MemoryTypeFilter::PREFER_DEVICE | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
				..Default::default()
			},
			vertices.iter().map(|v| *v)
		).map_err(|_| ())?;

		let indices = Buffer::from_iter(
			self.buffer_allocator.clone(),
			BufferCreateInfo {
				usage: BufferUsage::INDEX_BUFFER,
				..Default::default()
			},
			AllocationCreateInfo {
				memory_type_filter: MemoryTypeFilter::PREFER_DEVICE | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
				..Default::default()
			},
			indices.iter().map(|i| *i)
		).map_err(|_| ())?;

		let internal = MeshDataInternal { vertices, indices };
		self.mesh_data.insert(uuid, internal);

		return Ok(Arc::new(MeshData { uuid, render_engine: engine }))
	}

	fn drop_mesh_data(&mut self, uuid: Uuid) {
		self.mesh_data.remove(&uuid);
	}
}

