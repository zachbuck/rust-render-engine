
use uuid::Uuid;
use vulkano::{buffer::{Buffer, BufferContents, BufferCreateInfo, Subbuffer}, memory::allocator::AllocationCreateInfo};

use crate::RenderEngine;

#[repr(C)]
#[derive(BufferContents, vulkano::pipeline::graphics::vertex_input::Vertex)]
pub struct Vertex {
	#[format(R32G32B32_SFLOAT)]
	pub position: [f32; 3],

	#[format(R32G32B32_SFLOAT)]
	pub normal: [f32; 3],

	#[format(R32G32_SFLOAT)]
	pub uv: [f32; 2],
}

pub struct MeshData {
	pub(crate) uuid: Uuid,
}

#[derive(Clone)]
pub(crate) struct MeshDataInternal {
	pub(crate) vertices: Subbuffer<[Vertex]>,	
	pub(crate) indices: Subbuffer<[u16]>,
}

impl RenderEngine {
	pub fn create_mesh_data(&mut self, vertices: Vec<Vertex>, indices: Vec<u16>) -> MeshData {
		let uuid = Uuid::now_v7();

		let vertices = Buffer::from_iter(
			self.buffer_allocator.clone(), 
			BufferCreateInfo {
				usage: vulkano::buffer::BufferUsage::VERTEX_BUFFER,
				..Default::default()
			}, 
			AllocationCreateInfo {
				memory_type_filter: vulkano::memory::allocator::MemoryTypeFilter::PREFER_DEVICE | vulkano::memory::allocator::MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
				..Default::default()
			}, 
			vertices
		).unwrap();

		let indices = Buffer::from_iter(
			self.buffer_allocator.clone(),
			BufferCreateInfo {
				usage: vulkano::buffer::BufferUsage::INDEX_BUFFER,
				..Default::default()
			},
			AllocationCreateInfo {
				memory_type_filter: vulkano::memory::allocator::MemoryTypeFilter::PREFER_DEVICE | vulkano::memory::allocator::MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
				..Default::default()
			},
			indices
		).unwrap();

		let internal = MeshDataInternal {
			vertices: vertices,
			indices: indices
		};

		self.mesh_data.insert(uuid, internal);

		MeshData { 
			uuid: uuid
		}
	}
}