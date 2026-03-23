
use std::{
	any::Any, 
	sync::Arc
};

use uuid::Uuid;
use vulkano::{
	command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer}, 
	device::Queue, 
	sync::GpuFuture
};

use crate::{
	render_engine::{
		render_resources::RenderResources, 
		render_thread::RenderThread
	}, 
	renderable::Renderable
};

pub mod image_surface;

pub(crate) mod render_surface_command;

pub(crate) trait RenderSurface: Any {
	fn begin_rendering<'a>(&self, builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>) -> Result<&'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, ()>;
	fn render_renderable<'a>(&self, builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, renderable: &Box<dyn Renderable>, resources: &RenderResources) -> Result<&'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, ()>;
	fn end_rendering(&mut self, builder: AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, future: Box<dyn GpuFuture + Send>, queue: Arc<Queue>) -> Result<Box<dyn GpuFuture + Send>, ()>;

	fn as_any(&self) -> &dyn Any;
	#[expect(unused)]
	fn as_mut_any(&mut self) -> &mut dyn Any;
}

impl RenderThread {
	#[inline]
	pub(crate) fn get_render_surface(&self, uuid: &Uuid) -> Option<&dyn RenderSurface> {
		self.render_surfaces.get(uuid).map(|rs| rs.as_ref())
	}

	#[inline]
	pub(crate) fn get_mut_render_surface(&mut self, uuid: &Uuid) -> Option<&mut dyn RenderSurface> {
		self.render_surfaces.get_mut(uuid).map(|rs| rs.as_mut())
	}
}
