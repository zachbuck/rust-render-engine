
use std::sync::{
	Arc, 
	mpsc::{Sender, sync_channel}
};

use uuid::Uuid;
use vulkano::render_pass::RenderPass;

use crate::{
	render_engine::{
		RenderEngine, 
		engine_future::{EngineFuture, EngineFutureBuilder}, render_command::RenderEngineCommand
	}, 
	render_surface::{
		RenderSurface, RenderSurfaceInfo, image_surface::image_surface_commands::ImageSurfaceCommand, render_surface_command::RenderSurfaceCommand
	}
};

pub(crate) mod image_surface_commands;
pub(crate) mod image_surface_internal;

pub struct ImageSurface {
	uuid: Uuid,
	command_channel: Arc<Sender<RenderEngineCommand>>,

	render_pass: Arc<RenderPass>,

	pub x_size: u32,
	pub y_size: u32,
}

impl Drop for ImageSurface {
	fn drop(&mut self) {
		let _ = self.command_channel.send(
			ImageSurfaceCommand::DropImageSurface { 
				uuid: self.uuid
			}.into()
		);
	}
}

impl ImageSurface {
	pub fn new(render_engine: &RenderEngine, x_size: u32, y_size: u32) -> impl EngineFuture<Result<Arc<ImageSurface>, ()>> {
		let (send, recv) = sync_channel(1);

		render_engine.command_channel.send(
			ImageSurfaceCommand::CreateImageSurface { 
				channel: send, 
				x_size: x_size, 
				y_size: y_size, 
				command_channel: render_engine.command_channel.clone() 
			}.into()
		).unwrap();

		EngineFutureBuilder::new_channel(recv)
			.build()
	}

	pub fn render_all(&self) -> impl EngineFuture<Result<(), ()>> {
		let (send, recv) = sync_channel(1);

		self.command_channel.send(
			RenderSurfaceCommand::RenderRenderSurface { 
				sender: send, 
				uuid: self.uuid, 
			}.into()
		).unwrap();

		EngineFutureBuilder::new_channel(recv)
			.build()
	}

	pub fn get_image_surface_data(&self) -> impl EngineFuture<Result<Box<[u8]>, ()>> {
		let (func_send, func_recv) = sync_channel(1);
		let (send, recv) = sync_channel(1);

		self.command_channel.send(
			ImageSurfaceCommand::ReadImageSurfaceData { 
				uuid: self.uuid,

				func_send: func_send,
				fut_send: send,
			}.into()
		).unwrap();

		EngineFutureBuilder::new_channel(func_recv)
			.with_gpu_future(recv)
			.then_transform(Box::new(|f| f()))
			.build()
	}
}

impl RenderSurface for ImageSurface {}

impl RenderSurfaceInfo for ImageSurface {
	fn get_render_pass(&self) -> &Arc<RenderPass> { &self.render_pass }
	fn get_command_sender(&self) -> &Arc<Sender<RenderEngineCommand>> { &self.command_channel }
}

#[cfg(test)]
mod tests {
    use crate::{
		render_engine::{
			engine_future::EngineFuture,
			RenderEngine, 
			RenderEngineFlags
		}, 
		render_surface::image_surface::ImageSurface,
	};

	#[test]
	/// Ensure that `ImageSurface::new()` and `ImageSurface::drop()` are working as expected.
	fn new_image_surface() {
		let engine = RenderEngine::new("Image Surface Test", [0, 1, 0], RenderEngineFlags::empty()).unwrap();

		let image_surface = ImageSurface::new(&engine, 100, 100).wait().unwrap();
		drop(image_surface);
	}

	#[test]
	/// Ensure that `ImageSurface::get_image_surface_data()` is working as expected.
	fn get_image_surface_data() {
		let engine = RenderEngine::new("Image Surface Test", [0, 1, 0], RenderEngineFlags::empty()).unwrap();

		let image_surface = ImageSurface::new(&engine, 100, 100).wait().unwrap();

		image_surface.render_all().wait().unwrap();

		let data = image_surface.get_image_surface_data().wait().unwrap();
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
