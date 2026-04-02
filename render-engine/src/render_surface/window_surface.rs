
use std::sync::{
	Arc, 
	mpsc::{Receiver, Sender, TryRecvError, sync_channel},
};

use sdl2::{event::Event, video::Window};
use uuid::Uuid;
use vulkano::{render_pass::RenderPass, swapchain::Surface};

use crate::{
	render_engine::{
		RenderEngine, 
		engine_future::{EngineFuture, EngineFutureBuilder}, 
		render_command::RenderEngineCommand,
	}, 
	render_surface::{RenderSurface, RenderSurfaceInfo, render_surface_command::RenderSurfaceCommand, window_surface::window_surface_command::WindowSurfaceCommand},
};

pub(crate) mod window_surface_command;
pub(crate) mod window_surface_internal;

#[expect(dead_code)]
pub struct WindowSurface {
	uuid: Uuid,
	command_channel: Arc<Sender<RenderEngineCommand>>,
	event_channel: Receiver<Event>,

	window: Window,
	surface: Arc<Surface>,
	render_pass: Arc<RenderPass>,
}

impl WindowSurface {
	pub fn new(render_engine: &mut RenderEngine, width: u32, height: u32, title: &str) -> impl EngineFuture<Result<Arc<WindowSurface>, ()>> {
		let video = &render_engine.sdl_resources.as_ref().unwrap().video;
		let window = video.window(title, width, height).vulkan().build().unwrap();

		let surface = unsafe { Surface::from_window_ref(render_engine.instance.clone(), &window).unwrap() };

		let (send, recv) = sync_channel(1);
		let (event_send, event_recv) = sync_channel(render_engine.flags.event_buffer_size as usize);
		render_engine.event_senders.insert(window.id(), event_send);

		render_engine.command_channel.send(
			WindowSurfaceCommand::CreateWindowSurface { 
				sender: send, 
				command_channel: render_engine.command_channel.clone(), 
				surface: surface.clone(), 
				window_extent: window.size().into(),
			}.into()
		).unwrap();

		EngineFutureBuilder::new_channel(recv)
			.then_transform(Box::new(move |(uuid, command_channel, render_pass)| Ok(Arc::new(WindowSurface { uuid, command_channel, event_channel: event_recv, window: window, surface: surface, render_pass }))))
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

	pub fn poll_event(&self) -> Option<Event> {
		let result = self.event_channel.try_recv();
		match result {
			Ok(e) => return Some(e),
			Err(TryRecvError::Disconnected) => panic!(),
			Err(TryRecvError::Empty) => return None,
		}
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

impl RenderSurface for WindowSurface {}

impl RenderSurfaceInfo for WindowSurface {
	fn get_render_pass(&self) -> &Arc<RenderPass> { &self.render_pass }
	fn get_command_sender(&self) -> &Arc<Sender<RenderEngineCommand>> { &self.command_channel }
}
