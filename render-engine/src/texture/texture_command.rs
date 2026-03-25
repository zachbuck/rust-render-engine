
use std::sync::{
	Arc, 
	mpsc::SyncSender
};

use uuid::Uuid;
use vulkano::{
	buffer::{Buffer, BufferCreateInfo, BufferUsage}, 
	command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferToImageInfo}, 
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
		self, 
		GpuFuture, 
		future::FenceSignalFuture
	}
};

use crate::{
	render_engine::{
		RenderEngine, 
		render_command::RenderEngineCommand, 
		render_thread::RenderThread
	}, 
	texture::{
		Texture, 
		texture_internal::TextureInternal
	}
};

#[derive(Debug)]
pub(crate) enum TextureCommand {
	CreateTexture {
		send: SyncSender<Result<Arc<Texture>, ()>>,
		fut_send: SyncSender<Arc<FenceSignalFuture<Box<dyn GpuFuture + Send>>>>,

		x_size: u32,
		y_size: u32,
		data: Box<[u8]>,
		engine: Arc<RenderEngine>,
	},
	GetTextures {
		send: SyncSender<Result<Box<[Arc<Texture>]>, ()>>,
	},
	DropTexture {
		uuid: Uuid,
	}
}

impl Into<RenderEngineCommand> for TextureCommand {
	fn into(self) -> RenderEngineCommand {
		RenderEngineCommand::TextureCommand(self)
	}
}

impl RenderThread {
	pub(crate) fn process_texture_command(&mut self, command: TextureCommand) {
		match command {
			TextureCommand::CreateTexture { send, fut_send, x_size, y_size, data, engine } => { let _ = send.send(self.create_texture(fut_send, x_size, y_size, data, engine)); },
			TextureCommand::GetTextures { send } => { let _ = send.send(self.get_textures()); },
			TextureCommand::DropTexture { uuid } => self.drop_texture(uuid),
		}
	}

	fn create_texture(&mut self, fut_send: SyncSender<Arc<FenceSignalFuture<Box<dyn GpuFuture + Send>>>>, x_size: u32, y_size: u32, data: Box<[u8]>, engine: Arc<RenderEngine>) -> Result<Arc<Texture>, ()> {
		let uuid = Uuid::now_v7();

		let buffer = Buffer::from_iter(
			self.buffer_allocator.clone(), 
			BufferCreateInfo {
				usage: BufferUsage::TRANSFER_SRC,
				..Default::default()
			}, 
			AllocationCreateInfo {
				memory_type_filter: MemoryTypeFilter::HOST_SEQUENTIAL_WRITE | MemoryTypeFilter::PREFER_HOST,
				..Default::default()
			}, 
			data.iter().map(|b| *b),
		).map_err(|_| ())?;

		let image = Image::new(
			self.buffer_allocator.clone(), 
			ImageCreateInfo {
				image_type: ImageType::Dim2d,
				format: Format::R8G8B8A8_UNORM,
				extent: [x_size, y_size, 1],
				usage: ImageUsage::SAMPLED | ImageUsage::TRANSFER_DST,
				..Default::default()
			}, 
			AllocationCreateInfo {
				memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
				..Default::default()
			}
		).map_err(|_| ())?;

		let queue = self.get_transfer_queue();

		let mut builder = AutoCommandBufferBuilder::primary(
			self.command_allocator.clone(), 
			queue.queue_family_index(), 
			CommandBufferUsage::OneTimeSubmit,
		).map_err(|_| ())?;

		builder
			.copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(buffer, image.clone())).map_err(|_| ())?;

		let command_buffer = builder.build().map_err(|_| ())?;

		let mut future = self.transfer_future.take().unwrap();
		future.cleanup_finished();

		let future = Arc::new(future
			.then_execute(queue, command_buffer).map_err(|_| { self.transfer_future = Some(sync::now(self.device.clone()).boxed_send()); () })?.boxed_send()
			.then_signal_fence_and_flush().map_err(|_| { self.transfer_future = Some(sync::now(self.device.clone()).boxed_send()); () })?);

		let _ = fut_send.send(future.clone());

		self.transfer_future = Some(future.boxed_send());

		let reference = Arc::new(Texture { uuid, render_engine: engine, y_size, x_size });

		let image_view = ImageView::new_default(image.clone()).map_err(|_| ())?;

		let internal = TextureInternal { reference: Arc::downgrade(&reference), image: image_view };

		self.textures.insert(uuid, internal);

		return Ok(reference);
	}

	fn get_textures(&mut self) -> Result<Box<[Arc<Texture>]>, ()> {
		Ok(self.textures.values().filter_map(|t| t.reference.upgrade()).collect())
	}
	
	fn drop_texture(&mut self, uuid: Uuid) {
		self.textures.remove(&uuid);
	}
}
