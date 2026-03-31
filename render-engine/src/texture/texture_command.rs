
use std::sync::{
	Arc, 
	mpsc::{Sender, SyncSender}
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
		sampler::{Filter, Sampler, SamplerCreateInfo}, 
		view::ImageView,
	}, 
	memory::allocator::{AllocationCreateInfo, MemoryTypeFilter}, 
	sync::{
		self, 
		GpuFuture, 
		future::FenceSignalFuture,
	},
};

use crate::{
	macros::error_map, 
	render_engine::{
		render_command::RenderEngineCommand, 
		render_thread::RenderThread,
	}, 
	texture::{
		Texture, 
		texture_internal::TextureInternal,
	},
};

#[derive(Debug)]
pub(crate) enum TextureCommand {
	CreateTexture {
		fut_send: 			SyncSender<Option<Arc<FenceSignalFuture<Box<dyn GpuFuture + Send>>>>>,
		send: 				SyncSender<Result<Arc<Texture>, ()>>,

		x_size: 			u32,
		y_size: 			u32,
		data: 				Box<[u8]>,
		command_channel: 	Arc<Sender<RenderEngineCommand>>,
	},
	GetTextures {
		send: 				SyncSender<Result<Box<[Arc<Texture>]>, ()>>,
	},
	DropTexture {
		uuid: 				Uuid,
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
			TextureCommand::CreateTexture { send, fut_send, x_size, y_size, data, command_channel } => { 
				let result = self.create_texture(x_size, y_size, data, command_channel);
				let _ = fut_send.send(result.as_ref().map(|(_, b)| Some(b.clone())).unwrap_or(None));
				let _ = send.send(result.map(|(a, _)| a.clone()));
			},
			TextureCommand::GetTextures { send } => { 
				let _ = send.send(self.get_textures()); 
			},
			TextureCommand::DropTexture { uuid } => self.drop_texture(uuid),
		}
	}

	fn create_texture(&mut self, x_size: u32, y_size: u32, data: Box<[u8]>, command_channel: Arc<Sender<RenderEngineCommand>>) -> Result<(Arc<Texture>, Arc<FenceSignalFuture<Box<dyn GpuFuture + Send>>>), ()> {
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
		).map_err(error_map!())?;

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
		).map_err(error_map!())?;

		let queue = self.get_transfer_queue();

		let mut builder = AutoCommandBufferBuilder::primary(
			self.command_allocator.clone(), 
			queue.queue_family_index(), 
			CommandBufferUsage::OneTimeSubmit,
		).map_err(error_map!())?;

		builder
			.copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(buffer, image.clone())).map_err(error_map!())?;

		let command_buffer = builder.build().map_err(error_map!())?;

		let mut future = self.transfer_future.take().unwrap();
		future.cleanup_finished();

		let future = Arc::new(future
			.then_execute(queue, command_buffer).map_err(|_| { self.transfer_future = Some(sync::now(self.device.clone()).boxed_send()); () })?.boxed_send()
			.then_signal_fence_and_flush().map_err(|_| { self.transfer_future = Some(sync::now(self.device.clone()).boxed_send()); () })?);

		self.transfer_future = Some(future.clone().boxed_send());

		let reference = Arc::new(Texture { uuid, command_channel, y_size, x_size });

		let image_view = ImageView::new_default(image.clone()).map_err(error_map!())?;

		let sampler = Sampler::new(
			self.device.clone(), 
			SamplerCreateInfo {
				mag_filter: Filter::Nearest,
				min_filter: Filter::Nearest,
				..Default::default()
			}
		).map_err(error_map!())?;

		let internal = TextureInternal { 
			reference: Arc::downgrade(&reference), 
			image: image_view,
			sampler: sampler,
		};

		self.textures.insert(uuid, internal);

		return Ok((reference, future.clone()));
	}

	fn get_textures(&mut self) -> Result<Box<[Arc<Texture>]>, ()> {
		Ok(self.textures.values().filter_map(|t| t.reference.upgrade()).collect())
	}
	
	fn drop_texture(&mut self, uuid: Uuid) {
		self.textures.remove(&uuid);
	}
}
