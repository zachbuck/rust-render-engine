
use std::{
	any::Any, collections::HashMap, sync::Arc
};

use uuid::Uuid;
use vulkano::{
	command_buffer::{
		AutoCommandBufferBuilder, 
		PrimaryAutoCommandBuffer,
	}, 
	pipeline::Pipeline as _,
};

use crate::{
	macros::error_map, mesh_data::MeshData, pipeline::Pipeline, render_engine::{
		render_resources::RenderResources, 
		render_thread::RenderThread,
	}, renderable::{
		Renderable, 
		descriptor_set_data::DescriptorSetData,
	}
};

pub(crate) struct RenderObjectInternal {
	pub(crate) mesh: Arc<MeshData>,
	pub(crate) pipeline: Arc<Pipeline>,

	pub(crate) descriptor_data: Box<[DescriptorSetData]>,
}

impl Renderable for RenderObjectInternal {
	fn draw<'a>(&self, builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, resources: &RenderResources) -> Result<&'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, ()> {
		let mesh = resources.get_mesh_data(&self.mesh).ok_or(())?;
		let pipeline = resources.get_pipeline(&self.pipeline).ok_or(())?;

		mesh.bind(builder)?;
		pipeline.bind(builder)?;
		for descriptor_set in &self.descriptor_data {
			descriptor_set.bind(builder, pipeline.pipeline.layout())?;
		}

		unsafe {
			builder
				.draw_indexed(mesh.index_count(), 1, 0, 0, 0).map_err(error_map!())?;
		}

		return Ok(builder)
	}

	fn as_any(&self) -> &dyn Any { self }
	fn as_mut_any(&mut self) -> &mut dyn Any { self }
}

impl RenderThread {
	#[inline]
	pub(crate) fn get_mut_render_object<'a>(renderables: &'a mut HashMap<Uuid, Box<dyn Renderable>>, uuid: &Uuid) -> Option<&'a mut RenderObjectInternal> {
		renderables.get_mut(uuid)?.as_mut_any().downcast_mut()
	}
}
