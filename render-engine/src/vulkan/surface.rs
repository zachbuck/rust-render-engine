
use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer};

pub mod image_surface;

pub trait Surface {
	fn begin_rendering<'a>(&mut self, builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>) -> Result<&'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, ()>;
	fn end_rendering<'a>(&mut self, builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>) -> Result<&'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, ()>;
}


