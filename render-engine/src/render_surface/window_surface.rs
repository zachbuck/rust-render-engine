
use std::sync::{
	Arc, 
	mpsc::{Sender, sync_channel},
};

use sdl2::video::Window;
use uuid::Uuid;
use vulkano::swapchain::Surface;

use crate::{
	render_engine::{
		RenderEngine, 
		engine_future::{EngineFuture, EngineFutureType}, 
		render_command::RenderEngineCommand,
	}, 
	render_surface::{render_surface_command::RenderSurfaceCommand, window_surface::window_surface_command::WindowSurfaceCommand},
};

pub(crate) mod window_surface_command;
pub(crate) mod window_surface_internal;

#[expect(dead_code)]
pub struct WindowSurface {
	uuid: Uuid,
	command_channel: Arc<Sender<RenderEngineCommand>>,

	window: Window,
	surface: Arc<Surface>,
}

impl WindowSurface {
	pub fn new(render_engine: &RenderEngine, width: u32, height: u32, title: &str) -> Arc<WindowSurface> {
		let video = render_engine.sdl.as_ref().unwrap().video().unwrap();
		let window = video.window(title, width, height).vulkan().build().unwrap();

		let surface = unsafe { Surface::from_window_ref(render_engine.instance.clone(), &window).unwrap() };

		let (send, recv) = sync_channel(1);

		render_engine.command_channel.send(
			WindowSurfaceCommand::CreateWindowSurface { 
				sender: send, 
				command_channel: render_engine.command_channel.clone(), 
				surface: surface.clone(), 
				window_extent: window.size().into(),
			}.into()
		).unwrap();

		let (uuid, sender) = recv.recv().unwrap();
		Arc::new(WindowSurface {
			uuid: uuid,
			command_channel: sender,
			window: window,
			surface: surface,
		})
	}

	pub fn render_all(&self) -> EngineFuture<Result<(), ()>> {
		let (send, recv) = sync_channel(1);

		self.command_channel.send(
			RenderSurfaceCommand::RenderRenderSurface { 
				sender: send, 
				uuid: self.uuid 
			}.into()
		).unwrap();

		EngineFuture::new_single(recv)
	}
}

impl Drop for WindowSurface {
	fn drop(&mut self) {
		let (send, recv) = sync_channel(1);

		self.command_channel.send(
			WindowSurfaceCommand::DropWindowSurface {
				sender: send,
				uuid: self.uuid,
			}.into()
		).unwrap();

		recv.recv().unwrap();
	}
}
