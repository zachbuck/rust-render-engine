
use std::sync::Arc;

use uuid::Uuid;
use vulkano::{
	command_buffer::{
		AutoCommandBufferBuilder, 
		PrimaryAutoCommandBuffer, 
		RenderingAttachmentInfo, 
		RenderingInfo
	}, 
	device::Queue, 
	image::view::ImageView, 
	pipeline::graphics::viewport::{Scissor, Viewport}, 
	render_pass::{AttachmentLoadOp, AttachmentStoreOp}, 
	sync::{
		GpuFuture, 
		future::FenceSignalFuture
	}
};

use crate::{
	macros::error_map, render_engine::{
		render_resources::RenderResources, 
		render_thread::RenderThread
	}, render_surface::RenderSurface, renderable::Renderable
};

pub(crate) struct ImageSurfaceInternal {
	pub(crate) image: Arc<ImageView>,
	pub(super) operation_future: Option<Arc<FenceSignalFuture<Box<dyn GpuFuture + Send>>>>,
}

impl RenderSurface for ImageSurfaceInternal {
	fn begin_rendering<'a>(&self, builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>) -> Result<&'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, ()> {
		builder.begin_rendering(
			RenderingInfo {
				color_attachments: vec![
					Some(RenderingAttachmentInfo {
						load_op: AttachmentLoadOp::Clear,
						store_op: AttachmentStoreOp::Store,
						clear_value: Some([0.0, 0.0, 0.0, 1.0].into()),
						..RenderingAttachmentInfo::image_view(self.image.clone())
					})
				],
				..Default::default()
			}
		).map_err(error_map!())?;

		builder
			.set_scissor_with_count(vec![ Scissor { offset: [0, 0], extent: [self.image.image().extent()[0], self.image.image().extent()[1]] } ].into()).map_err(error_map!())?
			.set_viewport_with_count(vec![Viewport { offset: [0.0, 0.0], extent: [self.image.image().extent()[0] as f32, self.image.image().extent()[1] as f32], depth_range: 0.0..=1.0 }].into()).map_err(error_map!())?;

		Ok(builder)
	}

	fn render_renderable<'a>(&self, builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, renderable: &Box<dyn Renderable>, resources: &RenderResources) -> Result<&'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, ()> {
		renderable.draw(builder, resources)?;

		Ok(builder)
	}

	fn end_rendering(&mut self, mut builder: AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, future: Box<dyn GpuFuture + Send>, queue: Arc<Queue>) -> Result<Box<dyn GpuFuture + Send>, ()> {
		builder.end_rendering().map_err(error_map!())?;
		let buffer = builder.build().map_err(error_map!())?;

		let future = future
			.then_execute(queue.clone(), buffer).map_err(error_map!())?.boxed_send()
			.then_signal_fence_and_flush().map_err(error_map!())?;

		let future = Arc::new(future);
		self.operation_future = Some(future.clone());

		return Ok(future.boxed_send());
	}

	fn as_any(&self) -> &dyn std::any::Any { self }
	fn as_mut_any(&mut self) -> &mut dyn std::any::Any { self }
}

impl RenderThread {
	#[inline]
	pub(crate) fn get_image_surface(&self, uuid: &Uuid) -> Option<&ImageSurfaceInternal> {
		self.get_render_surface(uuid)?.as_any().downcast_ref()
	}
}