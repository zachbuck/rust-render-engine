
use std::sync::Arc;

use vulkano::{
	buffer::Subbuffer, 
	command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer}
};

use crate::{
	mesh_data::{MeshData, Vertex3D}, 
	render_engine::render_thread::RenderThread
};

#[derive(Debug)]
pub(crate) struct MeshDataInternal {
	pub(crate) vertices: Subbuffer<[Vertex3D]>,
	pub(crate) indices: Subbuffer<[u16]>,
}

impl MeshDataInternal {
	#[inline]
	pub(crate) fn bind<'a>(&self, builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>) -> Result<&'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, ()> {
		builder
			.bind_vertex_buffers(0, self.vertices.clone()).map_err(|_| ())?
			.bind_index_buffer(self.indices.clone()).map_err(|_| ())?;

		return Ok(builder)
	}

	#[inline]
	pub(crate) fn index_count(&self) -> u32 {
		self.indices.len() as u32
	}
}

impl RenderThread {
	#[inline]
	#[expect(dead_code)]
	pub(crate) fn get_mesh_data_internal(&self, reference: Arc<MeshData>) -> Option<&MeshDataInternal> { self.mesh_data.get(&reference.uuid) }

	#[inline]
	#[expect(dead_code)]
	pub(crate) fn get_mut_mesh_data_internal(&mut self, reference: Arc<MeshData>) -> Option<&mut MeshDataInternal> { self.mesh_data.get_mut(&reference.uuid) }
}
