
use std::sync::{
	Arc, 
	mpsc::{Sender, SyncSender}
};

use uuid::Uuid;
use vulkano::{
	buffer::{Buffer, BufferCreateInfo, BufferUsage}, command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, CopyImageToBufferInfo}, format::Format, image::{
		Image, ImageCreateInfo, ImageLayout, ImageType, ImageUsage, view::ImageView
	}, memory::allocator::{AllocationCreateInfo, MemoryTypeFilter}, render_pass::{AttachmentDescription, AttachmentLoadOp, AttachmentReference, AttachmentStoreOp, Framebuffer, FramebufferCreateInfo, RenderPass, RenderPassCreateInfo, SubpassDescription}, sync::{
		self, GpuFuture, future::FenceSignalFuture
	}
};

use crate::{
	macros::error_map, 
	render_engine::{
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
		channel: 			SyncSender<Result<Arc<ImageSurface>, ()>>,

		x_size: 			u32,
		y_size: 			u32,
		command_channel: 	Arc<Sender<RenderEngineCommand>>,
	},
	DropImageSurface {
		uuid: 				Uuid
	},

	ReadImageSurfaceData {
		uuid: 				Uuid,

		func_send: 			SyncSender<Box<dyn FnOnce() -> Result<Box<[u8]>, ()> + Send>>,
		fut_send: 			SyncSender<Option<Arc<FenceSignalFuture<Box<dyn GpuFuture + Send>>>>>,
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
			ImageSurfaceCommand::CreateImageSurface { channel, x_size, y_size, command_channel } => { 
				let _ = channel.send(self.create_image_surface(x_size, y_size, command_channel)); 
			},
			ImageSurfaceCommand::DropImageSurface { uuid } => self.drop_image_surface(uuid),
			ImageSurfaceCommand::ReadImageSurfaceData { uuid, func_send, fut_send } => { 
				let (func, fut) = self.read_image_surface_data(uuid);
				let _ = func_send.send(func);
				let _ = fut_send.send(fut);
			},
		}
	}

	fn create_image_surface(&mut self, x_size: u32, y_size: u32, command_channel: Arc<Sender<RenderEngineCommand>>) -> Result<Arc<ImageSurface>, ()> {
		let uuid = Uuid::now_v7();

		let render_pass = RenderPass::new(
			self.device.clone(), 
			RenderPassCreateInfo {
				attachments: vec![
					AttachmentDescription {
						format: Format::R8G8B8A8_UNORM,
						load_op: AttachmentLoadOp::Clear,
						store_op: AttachmentStoreOp::Store,
						final_layout: ImageLayout::ColorAttachmentOptimal,
						..Default::default()
					},
					AttachmentDescription {
						format: Format::D32_SFLOAT,
						load_op: AttachmentLoadOp::Clear,
						store_op: AttachmentStoreOp::DontCare,
						final_layout: ImageLayout::DepthStencilAttachmentOptimal,
						..Default::default()
					}
				],
				subpasses: vec![
					SubpassDescription {
						color_attachments: vec![
							Some(AttachmentReference {
								attachment: 0,
								layout: ImageLayout::ColorAttachmentOptimal,
								..Default::default()
							})
						],
						depth_stencil_attachment: Some(
							AttachmentReference {
								attachment: 1,
								layout: ImageLayout::DepthStencilAttachmentOptimal,
								..Default::default()
							}
						),
						..Default::default()
					}
				],
				..Default::default()
			}
		).map_err(error_map!())?;

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
		).map_err(error_map!())?;

		let image_view = ImageView::new_default(image).map_err(error_map!())?;

		let depth_image = Image::new(
			self.buffer_allocator.clone(), 
			ImageCreateInfo {
				image_type: ImageType::Dim2d,
				format: Format::D32_SFLOAT,
				extent: [x_size, y_size, 1],
				usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT,
				..Default::default()
			}, 
			AllocationCreateInfo {
				memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
				..Default::default()
			}
		).map_err(error_map!())?;

		let depth_image_view =ImageView::new_default(depth_image).map_err(error_map!())?;

		let framebuffer = Framebuffer::new(
			render_pass.clone(), 
			FramebufferCreateInfo {
				attachments: vec![image_view, depth_image_view],
				extent: [x_size, y_size],
				..Default::default()
			}
		).map_err(error_map!())?;

		let internal = Box::new(ImageSurfaceInternal {
			render_pass: render_pass.clone(),
			framebuffer: framebuffer,
			operation_future: None,
		});

		self.render_surfaces.insert(uuid, internal);

		Ok(Arc::new(ImageSurface {
			uuid: uuid,
			command_channel,

			render_pass: render_pass.clone(),

			x_size: x_size,
			y_size: y_size
		}))
	}

	fn drop_image_surface(&mut self, uuid: Uuid) {
		self.render_surfaces.remove(&uuid);
	}

	fn read_image_surface_data(&mut self, uuid: Uuid) ->  (Box<dyn FnOnce() -> Result<Box<[u8]>, ()> + Send>, Option<Arc<FenceSignalFuture<Box<dyn GpuFuture + Send>>>>) {
		let queue = self.get_transfer_queue();
		
		let image_surface = Self::get_mut_image_surface(&mut self.render_surfaces, &uuid).unwrap();

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
			(0..image_surface.framebuffer.attachments()[0].image().extent()[0] * image_surface.framebuffer.attachments()[0].image().extent()[1] * 4).map(|_| 0u8),
		);
		if result.is_err() { return (Box::new(|| Err(())), None); } 

		let buffer = result.unwrap();

		let result = AutoCommandBufferBuilder::primary(
			self.command_allocator.clone(), 
			queue.queue_family_index(), 
			CommandBufferUsage::OneTimeSubmit	
		);
		if result.is_err() { return (Box::new(|| Err(())), None); }

		let mut builder = result.unwrap();

		let result = builder.copy_image_to_buffer(CopyImageToBufferInfo::image_buffer(image_surface.framebuffer.attachments()[0].image().clone(), buffer.clone()));
		if result.is_err() { return (Box::new(|| Err(())), None) }
		let result = builder.build();
		if result.is_err() { return (Box::new(|| Err(())), None) }

		let mut future = self.transfer_future.take().unwrap();
		future.cleanup_finished();

		let result = future.then_execute(queue.clone(), result.unwrap()).map(|f| f.boxed_send());
		if result.is_err() { self.transfer_future = Some(sync::now(self.device.clone()).boxed_send()); return (Box::new(|| Err(())), None) }
		let future = result.unwrap();
		let result = future.then_signal_fence_and_flush().map(|f| Arc::new(f));
		if result.is_err() {  self.transfer_future = Some(sync::now(self.device.clone()).boxed_send()); return (Box::new(|| Err(())), None) }
		let future = result.unwrap();

		self.transfer_future = Some(future.clone().boxed_send());
		image_surface.operation_future = Some(future.clone());

		return (Box::new(move || {
				Ok(buffer.read().unwrap().to_owned().into_boxed_slice())
			}),
			None,
		)
	}
}
