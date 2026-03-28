
use std::any::Any;

use vulkano::command_buffer::{
	AutoCommandBufferBuilder, 
	PrimaryAutoCommandBuffer,
};

use crate::render_engine::render_resources::RenderResources;

pub mod descriptor_set_data;
pub mod render_object;

pub(crate) trait Renderable: Any {
	fn draw<'a>(&self, builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, resources: &RenderResources) -> Result<&'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, ()>;

	#[expect(unused)]
	fn as_any(&self) -> &dyn Any;
	fn as_mut_any(&mut self) -> &mut dyn Any;
}
