
use std::{collections::HashMap, sync::Weak};

use uuid::Uuid;
use vulkano::{
	buffer::Subbuffer, 
	command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer}
};

use crate::{
	macros::error_map, mesh_data::{MeshData, Vertex3D}, render_engine::render_thread::RenderThread 
};

#[derive(Debug)]
pub(crate) struct MeshDataInternal {
	pub(crate) reference: Weak<MeshData>,

	pub(crate) vertices: Subbuffer<[Vertex3D]>,
	pub(crate) indices: Subbuffer<[u16]>,
}

impl MeshDataInternal {
	#[inline]
	pub(crate) fn bind<'a>(&self, builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>) -> Result<&'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, ()> {
		builder
			.bind_vertex_buffers(0, self.vertices.clone()).map_err(error_map!())?
			.bind_index_buffer(self.indices.clone()).map_err(error_map!())?;

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
	pub(crate) fn get_mesh_data<'a>(mesh_data: &'a HashMap<Uuid, MeshDataInternal>, uuid: &Uuid) -> Option<&'a MeshDataInternal> {
		mesh_data.get(uuid)
	}

	#[inline]
	#[expect(dead_code)]
	pub(crate) fn get_mut_mesh_data<'a>(mesh_data: &'a mut HashMap<Uuid, MeshDataInternal>, uuid: &Uuid) -> Option<&'a mut MeshDataInternal> {
		mesh_data.get_mut(uuid)
	}
}
