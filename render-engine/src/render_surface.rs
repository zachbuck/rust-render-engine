
use std::{
	any::Any, 
	sync::Arc
};

use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer};

use crate::{
	render_engine::render_resources::RenderResources, 
	renderable::Renderable
};

pub mod image_surface;

pub(crate) mod render_surface_command;

pub(crate) trait RenderSurface: Any {
	fn begin_rendering<'a>(&self, builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>) -> Result<&'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, ()>;
	fn render_renderable<'a>(&self, builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, renderable: &Box<dyn Renderable>, resources: &RenderResources) -> Result<&'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, ()>;
	fn end_rendering(&self, builder: AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>) -> Result<Arc<PrimaryAutoCommandBuffer>, ()>;

	fn as_any(&self) -> &dyn Any;
	fn as_mut_any(&mut self) -> &mut dyn Any;
}
