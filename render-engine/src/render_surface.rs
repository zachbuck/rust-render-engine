
use std::{
	any::Any, collections::HashMap, sync::{Arc, mpsc::Sender}
};

use uuid::Uuid;
use vulkano::{
	command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer}, device::Queue, render_pass::RenderPass, sync::GpuFuture
};

use crate::{
	render_engine::{
		render_command::RenderEngineCommand, render_resources::RenderResources, render_thread::RenderThread
	}, 
	renderable::Renderable,
};

pub mod image_surface;
pub mod window_surface;

pub(crate) mod render_surface_command;

pub(crate) trait RenderSurfaceInternal: Any {
	fn begin_rendering<'a>(&mut self, builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>) -> Result<&'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, bool>;
	fn render_renderable<'a>(&self, builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, renderable: &Box<dyn Renderable>, resources: &RenderResources) -> Result<&'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, ()>;
	fn end_rendering(&mut self, builder: AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, future: Box<dyn GpuFuture + Send>, queue: Arc<Queue>) -> Result<Box<dyn GpuFuture + Send>, ()>;

	fn as_any(&self) -> &dyn Any;
	fn as_mut_any(&mut self) -> &mut dyn Any;
}

impl RenderThread {
	#[inline]
	pub(crate) fn get_render_surface<'a>(render_surfaces: &'a HashMap<Uuid, Box<dyn RenderSurfaceInternal>>, uuid: &Uuid) -> Option<&'a dyn RenderSurfaceInternal> {
		render_surfaces.get(uuid).map(|b| b.as_ref())
	}

	#[inline]
	pub(crate) fn get_mut_render_surface<'a>(render_surfaces: &'a mut HashMap<Uuid, Box<dyn RenderSurfaceInternal>>, uuid: &Uuid) -> Option<&'a mut dyn RenderSurfaceInternal> {
		render_surfaces.get_mut(uuid).map(|b| b.as_mut())
	}
}

#[allow(private_bounds)]
pub trait RenderSurface: RenderSurfaceInfo {  }

pub(crate) trait RenderSurfaceInfo {
	fn get_render_pass(&self) -> &Arc<RenderPass>;
	fn get_command_sender(&self) -> &Arc<Sender<RenderEngineCommand>>;
}

impl<T> RenderSurface for Arc<T> 
where T: RenderSurface,
{}

impl<T> RenderSurfaceInfo for Arc<T> 
where T: RenderSurfaceInfo
{
	fn get_render_pass(&self) -> &Arc<RenderPass> { self.as_ref().get_render_pass() }
	fn get_command_sender(&self) -> &Arc<Sender<RenderEngineCommand>> { self.as_ref().get_command_sender() }
}
