use std::sync::Arc;

use image::{ImageBuffer, Rgba};
use uuid::Uuid;
use vulkano::{
	buffer::{Buffer, BufferCreateInfo}, command_buffer::{AutoCommandBufferBuilder, CopyImageToBufferInfo, PrimaryAutoCommandBuffer, RenderingAttachmentInfo, RenderingInfo}, image::{
		Image, 
		ImageCreateInfo, 
		ImageUsage, 
		view::ImageView
	}, memory::allocator::AllocationCreateInfo, pipeline::graphics::viewport::Viewport, sync::{self, GpuFuture}
};

use crate::{
	RenderEngine, 
	render_surface::{RenderCall, RenderSurface}
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
		let buffer = self.builder.take().unwrap().build().unwrap();

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

		builder.set_viewport(0, viewports.into()).unwrap();
	}
}

impl RenderEngine {
	pub fn create_image_surface(&mut self, x_size: u32, y_size: u32) -> ImageSurfaceHandle {
		let uuid = Uuid::now_v7();

		let builder = AutoCommandBufferBuilder::primary(
			self.command_allocator.clone(), 
			self.graphics_queue.queue_family_index(), 
			vulkano::command_buffer::CommandBufferUsage::OneTimeSubmit,
		).unwrap();

		let image = Image::new(
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
		).unwrap();

		let image_view = ImageView::new_default(image).unwrap();

		let mut internal = ImageSurfaceInternal {
			builder: Some(builder),
			image: image_view,
		};
		internal.builder_initial_commands();

		self.render_surfaces.insert(uuid, RenderSurface::Image(internal));

		ImageSurfaceHandle { uuid: uuid }
	}

	pub fn image_surface_push_render_calls(&mut self, handle: ImageSurfaceHandle) {
		let uuid = handle.uuid;
		let render_surface = self.render_surfaces.get_mut(&uuid).unwrap();
		let RenderSurface::Image(image) = render_surface; {
			let builder = AutoCommandBufferBuilder::primary(
				self.command_allocator.clone(), 
				self.graphics_queue.queue_family_index(), 
				vulkano::command_buffer::CommandBufferUsage::OneTimeSubmit
			).unwrap();

			let buffer = image.build_buffer(builder);

			let gpu_future = self.graphics_operation
				.take().unwrap()
				.then_execute(self.graphics_queue.clone(), buffer).unwrap()
				.then_signal_fence_and_flush().unwrap();

			self.graphics_operation = Some(gpu_future.boxed());
		}
	}

	pub fn get_image_surface_data(&mut self, handle: ImageSurfaceHandle) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
		let render_surface = self.render_surfaces.get(&handle.uuid).unwrap();
		let RenderSurface::Image(image_surface) = render_surface;

		let buffer = Buffer::from_iter(
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
		).unwrap();

		let mut builder = AutoCommandBufferBuilder::primary(
			self.command_allocator.clone(), 
			self.transfer_queue.queue_family_index(), 
			vulkano::command_buffer::CommandBufferUsage::OneTimeSubmit
		).unwrap();

		builder.copy_image_to_buffer(CopyImageToBufferInfo::image_buffer(image_surface.image.image().clone(), buffer.clone())).unwrap();

		let command_buffer = builder.build().unwrap();

		let gpu_future = self.transfer_operation
			.take().unwrap()
			.then_execute(self.transfer_queue.clone(), command_buffer).unwrap()
			.then_signal_fence_and_flush().unwrap();

		gpu_future.wait(None).unwrap();

		let buffer_contents = (buffer.read().unwrap()).to_vec();
		let image = ImageBuffer::<Rgba<u8>, _>::from_raw(
			image_surface.image.image().extent()[0],
			image_surface.image.image().extent()[1],
			buffer_contents
		).unwrap();

		self.transfer_operation = Some(sync::now(self.device.clone()).boxed());

		return image;
	}
}