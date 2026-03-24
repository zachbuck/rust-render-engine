
use std::sync::{
	Arc, 
	mpsc::SyncSender
};

use uuid::Uuid;
use vulkano::{
	buffer::{Buffer, BufferCreateInfo, BufferUsage}, 
	command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, CopyImageToBufferInfo}, 
	format::Format, 
	image::{
		Image, 
		ImageCreateInfo, 
		ImageType, 
		ImageUsage, 
		view::ImageView
	}, 
	memory::allocator::{AllocationCreateInfo, MemoryTypeFilter}, 
	sync::{
		self, GpuFuture, future::FenceSignalFuture
	}
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

	ReadImageSurfaceData {
		uuid: Uuid,

		func_send: SyncSender<Box<dyn FnOnce() -> Result<Box<[u8]>, ()> + Send>>,
		fut_send: SyncSender<Arc<FenceSignalFuture<Box<dyn GpuFuture + Send>>>>,
	}
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
			ImageSurfaceCommand::ReadImageSurfaceData { uuid, func_send, fut_send } => { let _ = func_send.send(self.read_image_surface_data(uuid, fut_send)) ;},
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

	fn read_image_surface_data(&mut self, uuid: Uuid, fut_send: SyncSender<Arc<FenceSignalFuture<Box<dyn GpuFuture + Send>>>>) ->  Box<dyn FnOnce() -> Result<Box<[u8]>, ()> + Send> {
		let queue = self.get_transfer_queue();
		
		let image_surface = self.get_image_surface(&uuid).unwrap();

		let result = Buffer::from_iter(
			self.buffer_allocator.clone(),
			BufferCreateInfo {
				usage: BufferUsage::TRANSFER_DST,
				..Default::default()
			},
			AllocationCreateInfo {
				memory_type_filter: MemoryTypeFilter::PREFER_HOST | MemoryTypeFilter::HOST_RANDOM_ACCESS,
				..Default::default()
			},
			(0..image_surface.image.image().extent()[0] * image_surface.image.image().extent()[1] * 4).map(|_| 0u8),
		);
		if result.is_err() { return Box::new(|| Err(())); } 

		let buffer = result.unwrap();

		let result = AutoCommandBufferBuilder::primary(
			self.command_allocator.clone(), 
			queue.queue_family_index(), 
			CommandBufferUsage::OneTimeSubmit	
		);
		if result.is_err() { return Box::new(|| Err(())); }

		let mut builder = result.unwrap();

		let result = builder.copy_image_to_buffer(CopyImageToBufferInfo::image_buffer(image_surface.image.image().clone(), buffer.clone()));
		if result.is_err() { return Box::new(|| Err(())) }
		let result = builder.build();
		if result.is_err() { return Box::new(|| Err(())) }

		let mut future = self.transfer_future.take().unwrap();
		future.cleanup_finished();

		let result = future.then_execute(queue.clone(), result.unwrap()).map(|f| f.boxed_send());
		if result.is_err() { self.transfer_future = Some(sync::now(self.device.clone()).boxed_send()); return Box::new(|| Err(())) }
		let future = result.unwrap();
		let result = future.then_signal_fence_and_flush().map(|f| Arc::new(f));
		if result.is_err() {  self.transfer_future = Some(sync::now(self.device.clone()).boxed_send()); return Box::new(|| Err(())) }
		let future = result.unwrap();

		let _ = fut_send.send(future.clone());

		self.transfer_future = Some(future.boxed_send());

		return Box::new(move || {
			Ok(buffer.read().unwrap().to_owned().into_boxed_slice())
		})
	}
}
