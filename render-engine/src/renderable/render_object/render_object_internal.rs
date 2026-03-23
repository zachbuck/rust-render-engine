
use std::{
	any::Any, 
	sync::Arc
};

use vulkano::command_buffer::{
	AutoCommandBufferBuilder, 
	PrimaryAutoCommandBuffer
};

use crate::{
	mesh_data::MeshData, 
	pipeline::Pipeline, 
	render_engine::render_resources::RenderResources, 
	renderable::Renderable
};

pub(crate) struct RenderObjectInternal {
	pub(crate) mesh: Arc<MeshData>,
	pub(crate) pipeline: Arc<Pipeline>,
}

impl Renderable for RenderObjectInternal {
	fn draw<'a>(&self, builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, resources: &RenderResources) -> Result<&'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, ()> {
		let mesh = resources.get_mesh_data(&self.mesh).ok_or(())?;
		let pipeline = resources.get_pipeline(&self.pipeline).ok_or(())?;

		mesh.bind(builder)?;
		pipeline.bind(builder)?;

		unsafe {
			builder
				.draw_indexed(mesh.index_count(), 1, 0, 0, 0).map_err(|_| ())?;
		}

		return Ok(builder)
	}

	fn as_any(&self) -> &dyn Any { self }
	fn as_mut_any(&mut self) -> &mut dyn Any { self }
}
