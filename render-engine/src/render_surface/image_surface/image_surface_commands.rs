
use std::sync::{
	Arc, 
	mpsc::SyncSender
};

use uuid::Uuid;
use vulkano::{
	format::Format, 
	image::{
		Image, 
		ImageCreateInfo, 
		ImageType, 
		ImageUsage, 
		view::ImageView
	}, 
	memory::allocator::{AllocationCreateInfo, MemoryTypeFilter}
};

use crate::{
	render_engine::{
		RenderEngine, 
		render_command::RenderEngineCommand, 
		render_thread::RenderThread
	}, 
	render_surface::image_surface::{
		ImageSurface, 
		image_surface_internal::ImageSurfaceInternal
	}
};

#[derive(Debug)]
pub(crate) enum ImageSurfaceCommand {
	CreateImageSurface {
		channel: SyncSender<Result<Arc<ImageSurface>, ()>>,

		x_size: u32,
		y_size: u32,
		render_engine: Arc<RenderEngine>
	},
	DropImageSurface {
		uuid: Uuid
	},
}

impl Into<RenderEngineCommand> for ImageSurfaceCommand {
	fn into(self) -> RenderEngineCommand {
		RenderEngineCommand::ImageSurfaceCommand(self)
	}
}

impl RenderThread {
	pub(crate) fn process_image_surface_command(&mut self, command: ImageSurfaceCommand) {
		match command {
			ImageSurfaceCommand::CreateImageSurface { channel, x_size, y_size, render_engine } => { let _ = channel.send(self.create_image_surface(x_size, y_size, render_engine)); },
			ImageSurfaceCommand::DropImageSurface { uuid } => self.drop_image_surface(uuid),
		}
	}

	fn create_image_surface(&mut self, x_size: u32, y_size: u32, render_engine: Arc<RenderEngine>) -> Result<Arc<ImageSurface>, ()> {
		let uuid = Uuid::now_v7();

		let image = Image::new(
			self.buffer_allocator.clone(),
			ImageCreateInfo {
				image_type: ImageType::Dim2d,
				format: Format::R8G8B8A8_UNORM,
				extent: [x_size, y_size, 1],
				usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::TRANSFER_SRC,
				..Default::default()
			},
			AllocationCreateInfo {
				memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
				..Default::default()
			}
		).map_err(|_| ())?;

		let image_view = ImageView::new_default(image).map_err(|_| ())?;

		let internal = Box::new(ImageSurfaceInternal {
			image: image_view,
			operation_future: None,
		});

		self.render_surfaces.insert(uuid, internal);

		Ok(Arc::new(ImageSurface {
			uuid: uuid,
			render_engine: render_engine,
			x_size: x_size,
			y_size: y_size
		}))
	}

	fn drop_image_surface(&mut self, uuid: Uuid) {
		self.render_surfaces.remove(&uuid);
	}
}
