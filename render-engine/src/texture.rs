
use std::sync::{
	Arc, 
	mpsc::{Sender, sync_channel},
};

use uuid::Uuid;

use crate::{
	render_engine::{
		RenderEngine, 
		engine_future::{EngineFuture, EngineFutureBuilder}, 
		render_command::RenderEngineCommand,
	}, 
	texture::texture_command::TextureCommand,
};

pub(crate) mod texture_command;
pub(crate) mod texture_internal;

#[derive(Debug)]
pub struct Texture {
	pub(crate) uuid: Uuid,
	command_channel: Arc<Sender<RenderEngineCommand>>,

	pub y_size: u32,
	pub x_size: u32,
}

impl Drop for Texture {
	fn drop(&mut self) {
		let _ = self.command_channel.send(
			TextureCommand::DropTexture { 
				uuid: self.uuid,
			}.into()
		);
	}
}

impl Texture {
	pub fn new(render_engine: &RenderEngine, data: &[u8], x_size: u32, y_size: u32) -> impl EngineFuture<Result<Arc<Texture>, ()>> {
		let (fut_send, fut_recv) = sync_channel(1);
		let (send, recv) = sync_channel(1);
		
		render_engine.command_channel.send(
			TextureCommand::CreateTexture { 
				send, 
				fut_send, 
				x_size, 
				y_size, 
				data: data.to_owned().into_boxed_slice(), 
				command_channel: render_engine.command_channel.clone(),
			}.into()
		).unwrap();

		EngineFutureBuilder::new_channel(recv)
			.with_gpu_future(fut_recv)
			.build()
	}

	pub fn get_all(render_engine: &RenderEngine) -> impl EngineFuture<Result<Box<[Arc<Texture>]>, ()>> {
		let (send, recv) = sync_channel(1);

		render_engine.command_channel.send(
			TextureCommand::GetTextures { 
				send: send,
			}.into()
		).unwrap();

		EngineFutureBuilder::new_channel(recv)
			.build()
	}
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
		render_engine::{
			engine_future::EngineFuture,
			RenderEngine, 
			RenderEngineFlags
		}, 
		texture::Texture,
	};

	const TEXTURE_DATA: [u8; 40000] = [0u8; 100 * 100 * 4];

	#[test]
	/// Ensure that `Texture::new()` and `Texture::drop()` are working as expected.
	fn new_texture() {
		let engine = RenderEngine::new("Texture Test", [0, 1, 0], RenderEngineFlags::empty()).unwrap();

		let texture = Texture::new(&engine, &TEXTURE_DATA, 100, 100).wait().unwrap();

		drop(texture);

		let texture_list = Texture::get_all(&engine).wait().unwrap();

		assert!(texture_list.len() == 0);
	}

	#[test]
	/// Ensure that `Texture::get_all()` is working as expected.
	fn get_all() {
		let engine = RenderEngine::new("Texture Test", [0, 1, 0], RenderEngineFlags::empty()).unwrap();

		let texture = Texture::new(&engine, &TEXTURE_DATA, 100, 100).wait().unwrap();

		let texture_list = Texture::get_all(&engine).wait().unwrap();

		assert!(texture_list.len() == 1);
		assert!(Arc::ptr_eq(&texture, &texture_list[0]));
	}
}
