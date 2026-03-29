
use std::sync::{
	Arc, 
	mpsc::{Sender, sync_channel}
};

use uuid::Uuid;

use crate::{
	render_engine::{
		RenderEngine, 
		engine_future::EngineFuture, render_command::RenderEngineCommand
	}, 
	render_surface::{
		image_surface::image_surface_commands::ImageSurfaceCommand, 
		render_surface_command::RenderSurfaceCommand
	}
};

pub(crate) mod image_surface_commands;
pub(crate) mod image_surface_internal;

pub struct ImageSurface {
	uuid: Uuid,
	command_channel: Arc<Sender<RenderEngineCommand>>,

	pub x_size: u32,
	pub y_size: u32,
}

impl Drop for ImageSurface {
	fn drop(&mut self) {
		self.command_channel.send(
			ImageSurfaceCommand::DropImageSurface { 
				uuid: self.uuid
			}.into()
		).unwrap();
	}
}

impl ImageSurface {
	pub fn new(render_engine: &RenderEngine, x_size: u32, y_size: u32) -> EngineFuture<Result<Arc<ImageSurface>, ()>> {
		let (send, recv) = sync_channel(1);

		render_engine.command_channel.send(
			ImageSurfaceCommand::CreateImageSurface { 
				channel: send, 
				x_size: x_size, 
				y_size: y_size, 
				command_channel: render_engine.command_channel.clone() 
			}.into()
		).unwrap();

		EngineFuture::new_single(recv)
	}

	pub fn render_all(&self) -> EngineFuture<Result<(), ()>> {
		let (send, recv) = sync_channel(1);

		self.command_channel.send(
			RenderSurfaceCommand::RenderRenderSurface { 
				sender: send, 
				uuid: self.uuid, 
			}.into()
		).unwrap();

		EngineFuture::new_single(recv)
	}

	pub fn get_image_surface_data(&self) -> EngineFuture<Result<Box<[u8]>, ()>> {
		let (func_send, func_recv) = sync_channel(1);
		let (send, recv) = sync_channel(1);

		self.command_channel.send(
			ImageSurfaceCommand::ReadImageSurfaceData { 
				uuid: self.uuid,

				func_send: func_send,
				fut_send: send,
			}.into()
		).unwrap();

		return EngineFuture::new_function(func_recv)
			.with_wait_condition(recv.into());
	}
}

#[cfg(test)]
mod tests {
    use crate::{
		render_engine::{RenderEngine, RenderEngineCreateInfo}, 
		render_surface::image_surface::ImageSurface,
	};

	#[test]
	/// Ensure that `ImageSurface::new()` and `ImageSurface::drop()` are working as expected.
	fn new_image_surface() {
		let create_info = RenderEngineCreateInfo::new();
		let engine = RenderEngine::new(create_info).unwrap();

		let image_surface = ImageSurface::new(&engine, 100, 100).unwrap().unwrap();
		drop(image_surface);
	}

	#[test]
	/// Ensure that `ImageSurface::get_image_surface_data()` is working as expected.
	fn get_image_surface_data() {
		let create_info = RenderEngineCreateInfo::new();
		let engine = RenderEngine::new(create_info).unwrap();

		let image_surface = ImageSurface::new(&engine, 100, 100).unwrap().unwrap();

		image_surface.render_all().unwrap().unwrap();

		let data = image_surface.get_image_surface_data().unwrap().unwrap();
		assert!(data.len() == 100 * 100 * 4);
		for i in 0..100 * 100 * 4 {
			if i % 4 == 3 {
				assert!(data[i] == 0xFF);
			} else {
				assert!(data[i] == 0x00);
			}
		}
	}
}
