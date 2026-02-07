use std::sync::Arc;

use uuid::Uuid;
use vulkano::{
	command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer}, 
	image::{
		Image, 
		ImageCreateInfo, 
		ImageUsage, 
		view::ImageView
	}, 
	memory::allocator::AllocationCreateInfo
};

use crate::{
	RenderEngine, 
	render_surface::{RenderCall, RenderSurface}
};

pub struct ImageSurfaceHandle {
	uuid: Uuid,
}

pub(crate) struct ImageSurfaceInternal {
	uuid: Uuid,
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
		return buffer;
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

		let internal = ImageSurfaceInternal {
			uuid: uuid,
			builder: Some(builder),
			image: image_view,
		};

		self.render_surfaces.insert(uuid, RenderSurface::Image(internal));

		ImageSurfaceHandle { uuid: uuid }
	}
}