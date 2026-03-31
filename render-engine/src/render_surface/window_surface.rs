
use std::sync::{
	Arc, 
	mpsc::{Sender, sync_channel},
};

use sdl2::{event::Event, keyboard::Keycode, video::Window};
use uuid::Uuid;
use vulkano::swapchain::Surface;

use crate::{
	render_engine::{
		RenderEngine, 
		engine_future::{EngineFuture, EngineFutureBuilder}, 
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
	pub fn new(render_engine: &RenderEngine, width: u32, height: u32, title: &str) -> impl EngineFuture<Result<Arc<WindowSurface>, ()>> {
		let video = &render_engine.sdl_resources.as_ref().unwrap().video;
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

		EngineFutureBuilder::new_channel(recv)
			.then_transform(Box::new(move |(uuid, command_channel)| Ok(Arc::new(WindowSurface { uuid, command_channel, window: window.clone(), surface: surface.clone() }))))
			.build()
	}

	pub fn render_all(&self) -> impl EngineFuture<Result<(), ()>> {
		let (send, recv) = sync_channel(1);

		self.command_channel.send(
			RenderSurfaceCommand::RenderRenderSurface { 
				sender: send, 
				uuid: self.uuid 
			}.into()
		).unwrap();

		EngineFutureBuilder::new_channel(recv)
			.build()
	}

	pub fn should_close(render_engine: &mut RenderEngine) -> bool {
		let event_pump = &mut render_engine.sdl_resources.as_mut().unwrap().event_pump;
		for event in event_pump.poll_iter() {
			match event {
				Event::Quit{..} => return true,
				Event::KeyDown{keycode: Some(Keycode::Escape), ..} => return true,
				_ => continue,
			}
		}
		return false;
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
