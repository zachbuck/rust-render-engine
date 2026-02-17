
pub mod image_surface;
pub use image_surface::ImageSurfaceHandle as ImageSurface;
use vulkano::{
	command_buffer::AutoCommandBufferBuilder, 
	sync::GpuFuture
};

use crate::{
	RenderEngine, 
	render_surface::image_surface::ImageSurfaceInternal, 
	render_target::RenderCall, unwrap_result_or_none
};

pub(crate) enum RenderSurface {
	Image(ImageSurfaceInternal),
}

impl RenderSurface {
	pub(crate) fn process_render_queue(&mut self, render_target: &dyn RenderCall) {
		match self {
			RenderSurface::Image(image_surface) => image_surface.process_render_queue(render_target),
		}
	}
}

impl RenderEngine {
	pub fn push_render_calls(&mut self) -> Result<(), ()> {
		for (_, render_surface) in &mut self.render_surfaces {
			match render_surface {
				RenderSurface::Image(image) => {
					let builder = unwrap_result_or_none!(AutoCommandBufferBuilder::primary(
						self.command_allocator.clone(), 
						self.graphics_queue.queue_family_index(), 
						vulkano::command_buffer::CommandBufferUsage::OneTimeSubmit,
					));

					let buffer = image.build_buffer(builder);

					let gpu_future = self.graphics_operation
						.take().unwrap()
						.then_execute(self.graphics_queue.clone(), buffer).unwrap()
						.then_signal_fence_and_flush().unwrap();

					self.graphics_operation = Some(gpu_future.boxed());
				}
			}
		}

		Ok(())
	}
}