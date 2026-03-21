
use std::sync::{
	Arc, 
	mpsc::sync_channel
};

use uuid::Uuid;

use crate::{render_engine::{
	RenderEngine, 
	engine_future::EngineFuture
}, render_surface::image_surface::image_surface_commands::ImageSurfaceCommand};

pub(crate) mod image_surface_commands;
pub(crate) mod image_surface_internal;

pub struct ImageSurface {
	uuid: Uuid,
	render_engine: Arc<RenderEngine>,

	pub x_size: u32,
	pub y_size: u32,
}

impl Drop for ImageSurface {
	fn drop(&mut self) {
		self.render_engine.command_channel.send(
			ImageSurfaceCommand::DropImageSurface { 
				uuid: self.uuid
			}.into()
		).unwrap();
	}
}

impl ImageSurface {
	pub fn new(render_engine: Arc<RenderEngine>, x_size: u32, y_size: u32) -> EngineFuture<Result<Arc<ImageSurface>, ()>> {
		let (send, recv) = sync_channel(1);

		render_engine.command_channel.send(
			ImageSurfaceCommand::CreateImageSurface { 
				channel: send, 
				x_size: x_size, 
				y_size: y_size, 
				render_engine: render_engine.clone() 
			}.into()
		).unwrap();

		EngineFuture::new_single(recv)
	}
}
