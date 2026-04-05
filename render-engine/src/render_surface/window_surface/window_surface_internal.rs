
use std::sync::Arc;

use vulkano::{
	command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo, SubpassEndInfo}, 
	device::Queue, 
	format::{ClearValue, Format}, 
	image::{
		Image, 
		ImageCreateInfo,  
		ImageType, 
		ImageUsage, 
		view::ImageView,
	}, 
	memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator}, 
	pipeline::graphics::viewport::{Scissor, Viewport}, 
	render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass}, 
	swapchain::{Swapchain, SwapchainAcquireFuture, SwapchainPresentInfo, acquire_next_image}, 
	sync::{GpuFuture, future::FenceSignalFuture}
};

use crate::{
	macros::{count_passings, error_map}, 
	render_engine::render_resources::RenderResources, 
	render_surface::RenderSurfaceInternal, 
	renderable::Renderable,
};

pub(crate) struct WindowSurfaceInternal {
	pub(crate) render_pass: Arc<RenderPass>,
	pub(super) swapchain: Arc<Swapchain>,
	pub(super) framebuffers: Box<[(Arc<Framebuffer>, Option<Arc<FenceSignalFuture<Box<dyn GpuFuture + Send>>>>)]>,

	pub(super) acquire_future: Option<SwapchainAcquireFuture>,
	pub(super) image_index: Option<u32>,
	pub(super) suboptimal: bool,
}

impl WindowSurfaceInternal {
	pub(super) fn get_frame_buffers(images: &[Arc<Image>], render_pass: &Arc<RenderPass>, allocator: &Arc<StandardMemoryAllocator>) -> Box<[(Arc<Framebuffer>, Option<Arc<FenceSignalFuture<Box<dyn GpuFuture + Send>>>>)]> {
		let extent = images[0].extent();

		images.iter().map(|i| {
			let d = Image::new(
				allocator.clone(), 
				ImageCreateInfo {
					image_type: ImageType::Dim2d,
					format: Format::D32_SFLOAT,
					extent: extent,
					usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT,
					..Default::default()
				}, 
				AllocationCreateInfo {
					memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
					..Default::default()
				}
			).unwrap();

			(i, d)
		}).map(|(i, d)| {
			let iv = ImageView::new_default(i.clone()).unwrap();
			let dv = ImageView::new_default(d).unwrap();

			(iv, dv)
		}).map(|(i, d)| {
			let framebuffer = Framebuffer::new(
				render_pass.clone(), 
				FramebufferCreateInfo {
					attachments: vec![i, d],
					extent: [extent[0], extent[1]],
					..Default::default()
				}
			).unwrap();

			framebuffer
		}).map(|f|
			(f, None)
		).collect::<Vec<_>>().into_boxed_slice()
	}
}

impl RenderSurfaceInternal for WindowSurfaceInternal {
	fn begin_rendering<'a>(&mut self, builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>) -> Result<&'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, bool> {
		if self.suboptimal {
			todo!()
		}

		if self.acquire_future.is_none() {
			let (index, suboptimal, acquire_future) = acquire_next_image(self.swapchain.clone(), None).unwrap();
			self.image_index = Some(index);
			self.suboptimal = suboptimal;
			self.acquire_future = Some(acquire_future);
		}

		let (framebuffer, future) = &mut self.framebuffers[self.image_index.unwrap() as usize];

		if !future.as_ref().map(|f| f.is_signaled().unwrap()).unwrap_or(true) { return Err(false); }
		future.as_mut().map(|f| f.cleanup_finished());

		let extent = framebuffer.extent();

		builder.begin_render_pass(
			RenderPassBeginInfo {
				render_pass: self.render_pass.clone(),
				clear_values: vec![
					Some([0.0, 0.0, 0.0, 1.0].into()),
					Some(ClearValue::Depth(1.0)),
				],
				..RenderPassBeginInfo::framebuffer(framebuffer.clone())
			}, 
			SubpassBeginInfo::default()
		).unwrap();
			
		builder
			.set_scissor_with_count(vec![ Scissor { offset: [0, 0], extent: extent } ].into()).map_err(|_| true)?
			.set_viewport_with_count(vec![Viewport { offset: [0.0, 0.0], extent: [extent[0] as f32, extent[1] as f32], depth_range: 0.0..=1.0 }].into()).map_err(|_| true)?;
		
		return Ok(builder)
	}

	fn render_renderable<'a>(&self, builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, renderable: &Box<dyn Renderable>, resources: &RenderResources) -> Result<&'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, ()> {
		renderable.draw(builder, resources)
	}

	fn end_rendering(&mut self, mut builder: AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, future: Box<dyn GpuFuture + Send>, queue: Arc<Queue>) -> Result<Box<dyn GpuFuture + Send>, ()> {
		builder.end_render_pass(SubpassEndInfo::default()).unwrap();

		let command_buffer = builder.build().map_err(error_map!())?;

		let future = Arc::new(future
			.join(self.acquire_future.take().unwrap())
			.then_execute(queue.clone(), command_buffer).map_err(error_map!())?
			.then_swapchain_present(queue.clone(), SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), self.image_index.unwrap())).boxed_send()
			.then_signal_fence_and_flush().map_err(error_map!())?);
	
		self.framebuffers[self.image_index.unwrap() as usize].1 = Some(future.clone());

		return Ok(future.boxed_send())
	}

	fn as_any(&self) -> &dyn std::any::Any { return self }
	fn as_mut_any(&mut self) -> &mut dyn std::any::Any { return self }
}
