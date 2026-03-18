use std::sync::{
	Arc, 
	mpsc::{Sender, channel}
};

use uuid::Uuid;
use vulkano::{buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer}, memory::allocator::{AllocationCreateInfo, MemoryTypeFilter}};

use crate::render_engine::{EngineFuture, RenderEngine, RenderEngineCommand, RenderThread};

pub struct MeshData {
	uuid: Uuid,
	render_engine: Arc<RenderEngine>,
}

impl MeshData {
	pub fn new(render_engine: Arc<RenderEngine>, vertices: Vec<Vertex3D>, indices: Vec<u16>) -> EngineFuture<Result<Arc<Self>, ()>> {
		let (send, recv) = channel();

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

pub(crate) enum MeshDataCommand {
	CreateMeshData {
		sender: 	Sender<Result<Arc<MeshData>, ()>>,

		vertices:	Box<[Vertex3D]>,
		indices:	Box<[u16]>,
		engine: 	Arc<RenderEngine>,
	},
}

impl RenderThread {
	pub(crate) fn process_mesh_data_command(&mut self, command: MeshDataCommand) {
		match command {
			MeshDataCommand::CreateMeshData { sender, vertices, indices, engine } => sender.send(self.create_mesh_data(&vertices, &indices, engine)).unwrap(),
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
}

pub(crate) struct MeshDataInternal {
	vertices: Subbuffer<[Vertex3D]>,
	indices: Subbuffer<[u16]>,
}

#[repr(C)]
#[derive(BufferContents)]
#[derive(Clone, Copy)]
pub struct Vertex3D {
	pub pos: [f32; 3],
	pub norm: [f32; 3],
	pub uv: [f32; 2],
}