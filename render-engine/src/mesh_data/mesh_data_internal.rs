
use std::sync::Arc;

use vulkano::buffer::Subbuffer;

use crate::{
	mesh_data::{MeshData, Vertex3D}, 
	render_engine::render_thread::RenderThread
};

#[derive(Debug)]
pub(crate) struct MeshDataInternal {
	pub(crate) vertices: Subbuffer<[Vertex3D]>,
	pub(crate) indices: Subbuffer<[u16]>,
}

impl RenderThread {
	#[inline]
	#[expect(dead_code)]
	pub(crate) fn get_mesh_data_internal(&self, reference: Arc<MeshData>) -> Option<&MeshDataInternal> { self.mesh_data.get(&reference.uuid) }

	#[inline]
	#[expect(dead_code)]
	pub(crate) fn get_mut_mesh_data_internal(&mut self, reference: Arc<MeshData>) -> Option<&mut MeshDataInternal> { self.mesh_data.get_mut(&reference.uuid) }
}
