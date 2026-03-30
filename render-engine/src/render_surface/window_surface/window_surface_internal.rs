
use std::sync::Arc;

use vulkano::{
	command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer, RenderingAttachmentInfo, RenderingInfo}, device::Queue, image::{
		Image, 
		view::ImageView
	}, pipeline::graphics::viewport::{Scissor, Viewport}, render_pass::{AttachmentLoadOp, AttachmentStoreOp}, swapchain::{Swapchain, SwapchainAcquireFuture, SwapchainPresentInfo, acquire_next_image}, sync::GpuFuture
};

use crate::{
	macros::error_map, 
	render_engine::render_resources::RenderResources, 
	render_surface::RenderSurface, 
	renderable::Renderable,
};

pub(crate) struct WindowSurfaceInternal {
	pub(super) swapchain: Arc<Swapchain>,
	pub(super) images: Box<[Arc<ImageView>]>,

	pub(super) render_info: Option<RenderInfo>,
	pub(super) suboptimal: bool,
}

pub(super) struct RenderInfo {
	acquire_future: SwapchainAcquireFuture,
	image_index: u32,
}

impl WindowSurfaceInternal {
	pub(super) fn get_image_views(images: &[Arc<Image>]) -> Result<Box<[Arc<ImageView>]>, ()> {
		println!("Image Count: {:?}", images.len());
		let mut out = Vec::with_capacity(images.len());
		for i in 0..images.len(){ 
			out.push(ImageView::new_default(images[i].clone()).map_err(error_map!())?);
		}
		return Ok(out.into_boxed_slice());
	}
}

impl RenderSurface for WindowSurfaceInternal {
	fn begin_rendering<'a>(&mut self, builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>) -> Result<&'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, ()> {
		if self.suboptimal {
			todo!("MAKE THE WINDOW RECREATE THE SWAPCHAIN")
		}

		let (image_index, suboptimal, acquire_future) = acquire_next_image(self.swapchain.clone(), None).map_err(error_map!())?;
		self.render_info = Some(RenderInfo {
				acquire_future: acquire_future,
				image_index: image_index,
			});
		self.suboptimal = suboptimal;
		
		let image_view = &self.images[image_index as usize];

		builder
			.begin_rendering(RenderingInfo {
				color_attachments: vec![
					Some(RenderingAttachmentInfo {
						load_op: AttachmentLoadOp::Clear,
						store_op: AttachmentStoreOp::Store,
						clear_value: Some([0.0, 0.0, 0.0, 1.0].into()),
						..RenderingAttachmentInfo::image_view(image_view.clone())
					})
				],
				..Default::default()
			}).map_err(error_map!())?;

		builder
			.set_scissor_with_count(vec![ Scissor { offset: [0, 0], extent: [image_view.image().extent()[0], image_view.image().extent()[1]] } ].into()).map_err(error_map!())?
			.set_viewport_with_count(vec![Viewport { offset: [0.0, 0.0], extent: [image_view.image().extent()[0] as f32, image_view.image().extent()[1] as f32], depth_range: 0.0..=1.0 }].into()).map_err(error_map!())?;
			
		return Ok(builder)
	}

	fn render_renderable<'a>(&self, builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, renderable: &Box<dyn Renderable>, resources: &RenderResources) -> Result<&'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, ()> {
		renderable.draw(builder, resources)
	}

	fn end_rendering(&mut self, mut builder: AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, future: Box<dyn GpuFuture + Send>, queue: Arc<Queue>) -> Result<Box<dyn GpuFuture + Send>, ()> {
		builder.end_rendering().map_err(error_map!())?;
		let command_buffer = builder.build().map_err(error_map!())?;

		let render_info = self.render_info.take().unwrap();

		let future = future
			.join(render_info.acquire_future)
			.then_execute(queue.clone(), command_buffer).map_err(error_map!())?
			.then_swapchain_present(queue.clone(), SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), render_info.image_index))
			.then_signal_fence_and_flush().map_err(error_map!())?;

		return Ok(future.boxed_send())
	}

	fn as_any(&self) -> &dyn std::any::Any { return self }
	fn as_mut_any(&mut self) -> &mut dyn std::any::Any { return self }
}
