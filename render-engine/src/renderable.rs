
use std::any::Any;

use vulkano::command_buffer::{
	AutoCommandBufferBuilder, 
	PrimaryAutoCommandBuffer
};

use crate::render_engine::render_resources::RenderResources;

pub mod render_object;

pub(crate) trait Renderable: Any {
	fn draw<'a>(&self, builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, resources: &RenderResources) -> Result<&'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, ()>;

	fn as_any(&self) -> &dyn Any;
	fn as_mut_any(&mut self) -> &mut dyn Any;
}
