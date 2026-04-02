
use std::sync::{
	Arc, 
	mpsc::{Sender, SyncSender}
};

use uuid::Uuid;
use vulkano::{
	format::Format, 
	image::{ImageLayout, ImageUsage}, 
	render_pass::{AttachmentDescription, AttachmentLoadOp, AttachmentReference, AttachmentStoreOp, RenderPass, RenderPassCreateInfo, SubpassDescription}, 
	swapchain::{Surface, Swapchain, SwapchainCreateInfo}
};

use crate::{
	macros::error_map, 
	render_engine::{
		render_command::RenderEngineCommand, 
		render_thread::RenderThread,
	}, 
	render_surface::window_surface::{ 
		window_surface_internal::WindowSurfaceInternal,
	}
};

#[derive(Debug)]
pub(crate) enum WindowSurfaceCommand {
	CreateWindowSurface {
		sender: SyncSender<(Uuid, Arc<Sender<RenderEngineCommand>>, Arc<RenderPass>)>,

		command_channel: Arc<Sender<RenderEngineCommand>>,
		surface: Arc<Surface>,
		window_extent: [u32; 2]
	},
	DropWindowSurface {
		sender: SyncSender<()>,
		uuid: Uuid,
	},
}

impl RenderThread {
	pub(crate) fn process_window_surface_command(&mut self, command: WindowSurfaceCommand) {
		match command {
			WindowSurfaceCommand::CreateWindowSurface { sender, command_channel, surface, window_extent } => { 
				let _ = sender.send(self.create_window_surface(command_channel, surface, window_extent)); 
			},
			WindowSurfaceCommand::DropWindowSurface { sender, uuid } => { 
				let _ = sender.send(self.drop_window_surface(uuid)); 
			},
		}
	}

	fn create_window_surface(&mut self, command_channel: Arc<Sender<RenderEngineCommand>>, surface: Arc<Surface>, window_extent: [u32; 2]) -> (Uuid, Arc<Sender<RenderEngineCommand>>, Arc<RenderPass>) {
		let uuid = Uuid::now_v7();
		
		let surface_capabilites = self.device.physical_device()
			.surface_capabilities(&surface, Default::default()).map_err(error_map!()).unwrap();

		let (swapchain, images) = Swapchain::new(
			self.device.clone(), 
			surface.clone(), 
			SwapchainCreateInfo {
				min_image_count: surface_capabilites.min_image_count.max(2),
				image_format: Format::R8G8B8A8_UNORM,
				image_extent: window_extent,
				image_usage: ImageUsage::COLOR_ATTACHMENT,
				composite_alpha: surface_capabilites.supported_composite_alpha.into_iter().next().unwrap(),
				..Default::default()
			}
		).map_err(error_map!()).unwrap();

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
		).map_err(error_map!()).unwrap();

		let framebuffers = WindowSurfaceInternal::get_frame_buffers(&images, &render_pass, &self.buffer_allocator);

		let internal = WindowSurfaceInternal {
			render_pass: render_pass.clone(),
			swapchain: swapchain,
			framebuffers: framebuffers,
			acquire_future: None,
			image_index: None,
			suboptimal: false,
		};

		self.render_surfaces.insert(uuid, Box::new(internal));

		(uuid, command_channel, render_pass.clone())
	}

	fn drop_window_surface(&mut self, uuid: Uuid) {
		self.render_surfaces.remove(&uuid);
	}
}

impl Into<RenderEngineCommand> for WindowSurfaceCommand {
	fn into(self) -> RenderEngineCommand {
		RenderEngineCommand::WindowSurfaceCommand(self)
	}
}
