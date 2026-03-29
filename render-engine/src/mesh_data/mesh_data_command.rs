
use std::sync::{
	Arc, 
	mpsc::{Sender, SyncSender},
};

use uuid::Uuid;
use vulkano::{
	buffer::{Buffer, BufferCreateInfo, BufferUsage}, 
	memory::allocator::{AllocationCreateInfo, MemoryTypeFilter}
};

use crate::{
	macros::error_map, mesh_data::{
		MeshData, 
		Vertex3D, 
		mesh_data_internal::MeshDataInternal
	}, render_engine::{
		render_command::RenderEngineCommand, 
		render_thread::RenderThread
	}
};

#[derive(Debug)]
pub(crate) enum MeshDataCommand {
	CreateMeshData {
		sender: 			SyncSender<Result<Arc<MeshData>, ()>>,

		vertices:			Box<[Vertex3D]>,
		indices:			Box<[u16]>,
		command_channel: 	Arc<Sender<RenderEngineCommand>>,
	},
	GetMeshData {
		sender: 			SyncSender<Result<Box<[Arc<MeshData>]>, ()>>,
	},
	DropMeshData {
		uuid:				Uuid,
	}
}

impl Into<RenderEngineCommand> for MeshDataCommand {
	fn into(self) -> RenderEngineCommand {
		RenderEngineCommand::MeshDataCommand(self)
	}
}

impl RenderThread {
	pub(crate) fn process_mesh_data_command(&mut self, command: MeshDataCommand) {
		match command {
			MeshDataCommand::CreateMeshData { sender, vertices, indices, command_channel } => { let _ = sender.send(self.create_mesh_data(&vertices, &indices, command_channel)); },
			MeshDataCommand::GetMeshData { sender } => { let _ = sender.send(self.get_all_mesh_data()); },
			MeshDataCommand::DropMeshData { uuid } => self.drop_mesh_data(uuid),
		}
	}

	fn create_mesh_data(&mut self, vertices: &[Vertex3D], indices: &[u16], command_channel: Arc<Sender<RenderEngineCommand>>) -> Result<Arc<MeshData>, ()> {
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
		).map_err(error_map!())?;

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
		).map_err(error_map!())?;

		let reference = Arc::new(MeshData { uuid, command_channel });

		let internal = MeshDataInternal { reference: Arc::downgrade(&reference), vertices, indices };
		self.mesh_data.insert(uuid, internal);

		return Ok(reference);
	}

	fn get_all_mesh_data(&mut self) -> Result<Box<[Arc<MeshData>]>, ()> {
		Ok(self.mesh_data.values().filter_map(|i| i.reference.upgrade()).collect())
	}

	fn drop_mesh_data(&mut self, uuid: Uuid) {
		self.mesh_data.remove(&uuid);
	}
}

