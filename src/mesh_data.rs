
use uuid::Uuid;
use vulkano::buffer::{BufferContents, Subbuffer};

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
	uuid: Uuid,

	pub(crate) vertices: Subbuffer<[Vertex]>,	
	pub(crate) indices: Subbuffer<[u16]>,
}

impl RenderEngine {
	
}