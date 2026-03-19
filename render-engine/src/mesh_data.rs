
use std::sync::{
	Arc, 
	mpsc::{SyncSender, sync_channel}
};

use uuid::Uuid;
use vulkano::{
	buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer}, 
	memory::allocator::{AllocationCreateInfo, MemoryTypeFilter}, pipeline::graphics::vertex_input::Vertex
};

use crate::render_engine::{EngineFuture, RenderEngine, RenderEngineCommand, RenderThread};

#[derive(Debug)]
pub struct MeshData {
	uuid: Uuid,
	render_engine: Arc<RenderEngine>,
}

impl MeshData {
	pub fn new(render_engine: Arc<RenderEngine>, vertices: Vec<Vertex3D>, indices: Vec<u16>) -> EngineFuture<Result<Arc<Self>, ()>> {
		let (send, recv) = sync_channel(1);

		render_engine.command_channel.send(
			RenderEngineCommand::MeshDataCommand(
				MeshDataCommand::CreateMeshData {
					sender: send,

					vertices: vertices.into_boxed_slice(),
					indices: indices.into_boxed_slice(),
					engine: render_engine.clone(),
				}
			)
		).unwrap();

		return EngineFuture::new(recv);
	}
}

impl Drop for MeshData {
	fn drop(&mut self) {
		self.render_engine.command_channel.send(
			RenderEngineCommand::MeshDataCommand(
				MeshDataCommand::DropMeshData { 
					uuid: self.uuid 
				}
			)
		).unwrap();
	}
}

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

impl RenderThread {
	#[inline]
	pub(crate) fn get_mesh_data_internal(&self, reference: Arc<MeshData>) -> Option<&MeshDataInternal> { self.mesh_data.get(&reference.uuid) }
	#[inline]
	pub(crate) fn get_mut_mesh_data_internal(&mut self, reference: Arc<MeshData>) -> Option<&mut MeshDataInternal> { self.mesh_data.get_mut(&reference.uuid) }
}

#[derive(Debug)]
pub(crate) struct MeshDataInternal {
	vertices: Subbuffer<[Vertex3D]>,
	indices: Subbuffer<[u16]>,
}

#[repr(C)]
#[derive(BufferContents, Vertex)]
#[derive(Clone, Copy)]
#[derive(Debug)]
pub struct Vertex3D {
	#[format(R32G32B32_SFLOAT)]
	pub position: [f32; 3],
	#[format(R32G32B32_SFLOAT)]
	pub normal: [f32; 3],
	#[format(R32G32_SFLOAT)]
	pub uv: [f32; 2],
}