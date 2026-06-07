
use std::sync::Arc;

use uuid::Uuid;
use vulkano::{
	command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo, SubpassEndInfo}, 
	format::{ClearValue, Format}, 
	render_pass::{Framebuffer, RenderPass}, 
	sync::{
		GpuFuture, 
		future::FenceSignalFuture,
	},
};

use crate::{
	interface::engine_command::ImageSurfaceCommand,
	vulkan::{
		render_engine::RenderEngine, 
		surface::Surface,
	}
};

struct ImageSurface {
	render_pass: Arc<RenderPass>,
	framebuffer: Arc<Framebuffer>,
	operation_future: Option<Arc<FenceSignalFuture<Box<dyn GpuFuture + Send>>>>
}

impl Surface for ImageSurface {
	fn begin_rendering<'a>(&mut self, builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>) -> Result<&'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, ()> {
		if self.operation_future.as_ref().map(|f| f.is_signaled().unwrap()).unwrap_or(true) { return Err(()) }
		self.operation_future.as_mut().map(|f| f.cleanup_finished());

		builder.begin_render_pass(
			RenderPassBeginInfo {
				clear_values: vec![
					Some(ClearValue::Float([0.0, 0.0, 0.0, 1.0])),
					Some(ClearValue::Depth(1.0)),
				],
				..RenderPassBeginInfo::framebuffer(self.framebuffer.clone())
			}, 
			SubpassBeginInfo::default(),
		).map_err(|_| ())?;

		return Ok(builder)
	}

	fn end_rendering<'a>(&mut self, builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>) -> Result<&'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, ()> {
		builder.end_render_pass(
			SubpassEndInfo::default()
		).map_err(|_| ())?;

		return Ok(builder)
	}
}

impl RenderEngine {
	pub fn process_image_surface_command(&mut self, command: Box<ImageSurfaceCommand>) {
		match *command {
			ImageSurfaceCommand::CreateImageSurface { dimensions, vulkan_format, response } => response.send(self.create_image_surface(dimensions, vulkan_format)),
			ImageSurfaceCommand::DropImageSurface { uuid } => self.drop_image_surface(uuid),
		}
	}

	fn create_image_surface(&mut self, dimensions: [u32; 2], format: Format) -> Result<(Uuid,), ()> {
		println!("ImageSurface::new");

		let uuid = Uuid::now_v7();

		let image_surface = ImageSurface {
			render_pass: 		todo!(),
			framebuffer: 		todo!(),
			operation_future: 	todo!(),
		};
		self.surfaces.insert(uuid, Box::new(image_surface));

		Ok((uuid,))
	}

	fn drop_image_surface(&mut self, uuid: Uuid) -> () {
		println!("ImageSurface::drop");

		self.surfaces.remove(&uuid);
	}
}
