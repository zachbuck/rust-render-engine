
use std::sync::{Arc, mpsc::{Sender, SyncSender}};

use uuid::Uuid;
use vulkano::{
	format::Format, image::ImageUsage, swapchain::{Surface, Swapchain, SwapchainCreateInfo}
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
		sender: SyncSender<(Uuid, Arc<Sender<RenderEngineCommand>>)>,

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

	fn create_window_surface(&mut self, command_channel: Arc<Sender<RenderEngineCommand>>, surface: Arc<Surface>, window_extent: [u32; 2]) -> (Uuid, Arc<Sender<RenderEngineCommand>>) {
		let uuid = Uuid::now_v7();
		
		let surface_capabilites = self.device.physical_device()
			.surface_capabilities(&surface, Default::default()).map_err(error_map!()).unwrap();

		let (image_format, _) = *self.device.physical_device()
			.surface_formats(&surface, Default::default()).map_err(error_map!()).unwrap().iter().find(|(f, _)|  *f == Format::R8G8B8A8_UNORM).unwrap();

		let (swapchain, images) = Swapchain::new(
			self.device.clone(), 
			surface, 
			SwapchainCreateInfo {
				min_image_count: surface_capabilites.min_image_count.max(2),
				image_format: image_format,
				image_extent: window_extent,
				image_usage: ImageUsage::COLOR_ATTACHMENT,
				composite_alpha: surface_capabilites.supported_composite_alpha.into_iter().next().unwrap(),
				..Default::default()
			}
		).map_err(error_map!()).unwrap();

		let images_views = WindowSurfaceInternal::get_image_views(&images).unwrap();

		let internal = WindowSurfaceInternal {
			swapchain: swapchain,
			images: images_views,
			render_info: None,
			suboptimal: false,
		};

		self.render_surfaces.insert(uuid, Box::new(internal));

		(uuid, command_channel)
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
