
use std::{
	any::Any, collections::HashMap, sync::Arc
};

use uuid::Uuid;
use vulkano::{
	command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer}, 
	device::Queue, 
	sync::GpuFuture,
};

use crate::{
	render_engine::{
		render_resources::RenderResources, 
		render_thread::RenderThread
	}, 
	renderable::Renderable,
};

pub mod image_surface;

pub(crate) mod render_surface_command;

pub(crate) trait RenderSurface: Any {
	fn begin_rendering<'a>(&self, builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>) -> Result<&'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, ()>;
	fn render_renderable<'a>(&self, builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, renderable: &Box<dyn Renderable>, resources: &RenderResources) -> Result<&'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, ()>;
	fn end_rendering(&mut self, builder: AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, future: Box<dyn GpuFuture + Send>, queue: Arc<Queue>) -> Result<Box<dyn GpuFuture + Send>, ()>;

	fn as_any(&self) -> &dyn Any;
	fn as_mut_any(&mut self) -> &mut dyn Any;
}

impl RenderThread {
	#[inline]
	pub(crate) fn get_render_surface<'a>(render_surfaces: &'a HashMap<Uuid, Box<dyn RenderSurface>>, uuid: &Uuid) -> Option<&'a dyn RenderSurface> {
		render_surfaces.get(uuid).map(|b| b.as_ref())
	}

	#[inline]
	pub(crate) fn get_mut_render_surface<'a>(render_surfaces: &'a mut HashMap<Uuid, Box<dyn RenderSurface>>, uuid: &Uuid) -> Option<&'a mut dyn RenderSurface> {
		render_surfaces.get_mut(uuid).map(|b| b.as_mut())
	}
}
