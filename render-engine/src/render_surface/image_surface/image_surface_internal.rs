
use std::sync::Arc;

use vulkano::{
	command_buffer::{
		AutoCommandBufferBuilder, 
		PrimaryAutoCommandBuffer, 
		RenderingAttachmentInfo, 
		RenderingInfo
	}, 
	device::Queue, 
	image::view::ImageView, 
	render_pass::{AttachmentLoadOp, AttachmentStoreOp}, 
	sync::{
		GpuFuture, 
		future::FenceSignalFuture
	}
};

use crate::{
	render_engine::{
		render_resources::RenderResources, 
		render_thread::RenderThread
	}, 
	render_surface::{
		RenderSurface, 
		image_surface::ImageSurface
	}, 
	renderable::Renderable
};

pub(crate) struct ImageSurfaceInternal {
	pub(crate) image: Arc<ImageView>,
	pub(super) operation_future: Option<Arc<FenceSignalFuture<Box<dyn GpuFuture>>>>,
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
		).map_err(|_| ())?;

		Ok(builder)
	}

	fn render_renderable<'a>(&self, builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, renderable: &Box<dyn Renderable>, resources: &RenderResources) -> Result<&'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, ()> {
		renderable.draw(builder, resources)?;

		Ok(builder)
	}

	fn end_rendering(&mut self, mut builder: AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, future: Box<dyn GpuFuture>, queue: Arc<Queue>) -> Result<Box<dyn GpuFuture>, ()> {
		builder.end_rendering().map_err(|_| ())?;
		let buffer = builder.build().map_err(|_| ())?;

		let future = future
			.then_execute(queue.clone(), buffer).map_err(|_| ())?.boxed()
			.then_signal_fence_and_flush().map_err(|_| ())?;

		let future = Arc::new(future);
		self.operation_future = Some(future.clone());

		return Ok(future.boxed());
	}

	fn as_any(&self) -> &dyn std::any::Any { self }
	fn as_mut_any(&mut self) -> &mut dyn std::any::Any { self }
}

impl RenderThread {
	#[inline]
	#[expect(dead_code)]
	pub(crate) fn get_image_surface(&self, reference: Arc<ImageSurface>) -> Option<&ImageSurfaceInternal> {
		self.render_surfaces.get(&reference.uuid)?.as_any().downcast_ref()
	}

	#[inline]
	#[expect(dead_code)]
	pub(crate) fn get_mut_image_surface(&mut self, reference: Arc<ImageSurface>) -> Option<&mut ImageSurfaceInternal> {
		self.render_surfaces.get_mut(&reference.uuid)?.as_mut_any().downcast_mut()
	}
}
