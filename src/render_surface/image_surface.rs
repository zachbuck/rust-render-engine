use std::sync::Arc;

use image::{ImageBuffer, Rgba};
use uuid::Uuid;
use vulkano::{
	buffer::{Buffer, BufferCreateInfo}, 
	command_buffer::{AutoCommandBufferBuilder, CopyImageToBufferInfo, PrimaryAutoCommandBuffer, RenderingAttachmentInfo, RenderingInfo}, 
	image::{
		Image, 
		ImageCreateInfo, 
		ImageUsage, 
		view::ImageView
	}, 
	memory::allocator::AllocationCreateInfo, 
	pipeline::graphics::viewport::Viewport, 
	sync::{self, GpuFuture}
};

use crate::{
	RenderEngine, 
	render_surface::{RenderCall, RenderSurface}, 
	unwrap_option_or_none, 
	unwrap_result_or_none
};

pub struct ImageSurfaceHandle {
	uuid: Uuid,
}

pub(crate) struct ImageSurfaceInternal {
	builder: Option<AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>>,

	image: Arc<ImageView>,
}

impl ImageSurfaceInternal {
	pub(super) fn process_render_queue(&mut self, render_target: &dyn RenderCall) {
		render_target.render_call(&mut self.builder.as_mut().unwrap());
	}

	pub(super) fn build_buffer(&mut self, new_builder: AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>) -> Arc<PrimaryAutoCommandBuffer> {
		let mut builder = self.builder.take().unwrap();
		builder.end_rendering().unwrap();

		let buffer = builder.build().unwrap();

		self.builder = Some(new_builder);
		self.builder_initial_commands();

		return buffer;
	}

	fn builder_initial_commands(&mut self) {
		let builder = self.builder.as_mut().unwrap();

		builder.begin_rendering(RenderingInfo {
			color_attachments: vec![Some(RenderingAttachmentInfo {
				load_op: vulkano::render_pass::AttachmentLoadOp::Clear,
				store_op: vulkano::render_pass::AttachmentStoreOp::Store,
				clear_value: Some([0.0, 0.0, 0.0, 1.0].into()),
				..RenderingAttachmentInfo::image_view(self.image.clone())
			})],
			..Default::default()
		}).unwrap();
		
		let viewports = vec![Viewport {
			offset: [0.0, 0.0],
			extent: [self.image.image().extent()[0] as f32, self.image.image().extent()[1] as f32],
			depth_range: 0.0..=1.0,
		}];

		builder.set_viewport_with_count(viewports.into()).unwrap();
	}
}

impl RenderEngine {
	pub fn create_image_surface(&mut self, x_size: u32, y_size: u32) -> Result<ImageSurfaceHandle, ()> {
		let uuid = Uuid::now_v7();

		let builder = unwrap_result_or_none!(AutoCommandBufferBuilder::primary(
				self.command_allocator.clone(), 
				self.graphics_queue.queue_family_index(), 
				vulkano::command_buffer::CommandBufferUsage::OneTimeSubmit,
			));

		let image = unwrap_result_or_none!(Image::new(
			self.buffer_allocator.clone(),
			ImageCreateInfo {
				image_type: vulkano::image::ImageType::Dim2d,
				format: vulkano::format::Format::R8G8B8A8_UNORM,
				extent: [x_size, y_size, 1],
				usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::TRANSFER_DST | ImageUsage::TRANSFER_SRC,
				..Default::default()
			},
			AllocationCreateInfo {
				memory_type_filter: vulkano::memory::allocator::MemoryTypeFilter::PREFER_DEVICE,
				..Default::default()
			}
		));

		let image_view = unwrap_result_or_none!(ImageView::new_default(image));

		let mut internal = ImageSurfaceInternal {
			builder: Some(builder),
			image: image_view,
		};
		internal.builder_initial_commands();

		self.render_surfaces.insert(uuid, RenderSurface::Image(internal));

		Ok(ImageSurfaceHandle { uuid: uuid })
	}

	pub fn image_surface_push_render_calls(&mut self, handle: ImageSurfaceHandle) -> Result<(), ()> {
		let uuid = handle.uuid;
		let render_surface = unwrap_option_or_none!(self.render_surfaces.get_mut(&uuid));

		let RenderSurface::Image(image) = render_surface; {
			let builder = unwrap_result_or_none!(AutoCommandBufferBuilder::primary(
				self.command_allocator.clone(), 
				self.graphics_queue.queue_family_index(), 
				vulkano::command_buffer::CommandBufferUsage::OneTimeSubmit
			));

			let buffer = image.build_buffer(builder);

			let gpu_future = self.graphics_operation
				.take().unwrap()
				.then_execute(self.graphics_queue.clone(), buffer).unwrap()
				.then_signal_fence_and_flush().unwrap();

			self.graphics_operation = Some(gpu_future.boxed());
		};

		return Ok(());
	}

	pub fn get_image_surface_data(&mut self, handle: ImageSurfaceHandle) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, ()> {
		let render_surface = unwrap_option_or_none!(self.render_surfaces.get(&handle.uuid));
		let RenderSurface::Image(image_surface) = render_surface;

		let buffer = unwrap_result_or_none!(Buffer::from_iter(
			self.buffer_allocator.clone(), 
			BufferCreateInfo {
				usage: vulkano::buffer::BufferUsage::TRANSFER_DST,
				..Default::default()
			}, 
			AllocationCreateInfo {
				memory_type_filter: vulkano::memory::allocator::MemoryTypeFilter::PREFER_HOST | vulkano::memory::allocator::MemoryTypeFilter::HOST_RANDOM_ACCESS,
				..Default::default()
			}, 
			(0..1024 * 1024 * 4).map(|_| 0u8),
		));

		let mut builder = unwrap_result_or_none!(AutoCommandBufferBuilder::primary(
			self.command_allocator.clone(), 
			self.transfer_queue.queue_family_index(), 
			vulkano::command_buffer::CommandBufferUsage::OneTimeSubmit
		));

		unwrap_result_or_none!(builder.copy_image_to_buffer(CopyImageToBufferInfo::image_buffer(image_surface.image.image().clone(), buffer.clone())));

		let command_buffer = unwrap_result_or_none!(builder.build());

		let gpu_future = self.transfer_operation
			.take().unwrap()
			.then_execute(self.transfer_queue.clone(), command_buffer).unwrap()
			.then_signal_fence_and_flush().unwrap();

		gpu_future.wait(None).unwrap();

		let buffer_contents = (unwrap_result_or_none!(buffer.read())).to_vec();
		let image = unwrap_option_or_none!(ImageBuffer::<Rgba<u8>, _>::from_raw(
			image_surface.image.image().extent()[0],
			image_surface.image.image().extent()[1],
			buffer_contents
		));

		self.transfer_operation = Some(sync::now(self.device.clone()).boxed());

		return Ok(image);
	}
}