
use std::sync::{
	Arc, 
	mpsc::sync_channel
};

use uuid::Uuid;

use crate::{
	render_engine::{
		RenderEngine, 
		engine_future::EngineFuture
	}, 
	texture::texture_command::TextureCommand
};

pub(crate) mod texture_command;
pub(crate) mod texture_internal;

#[derive(Debug)]
pub struct Texture {
	pub(crate) uuid: Uuid,
	render_engine: Arc<RenderEngine>,

	pub y_size: u32,
	pub x_size: u32,
}

impl Drop for Texture {
	fn drop(&mut self) {
		self.render_engine.command_channel.send(
			TextureCommand::DropTexture { 
				uuid: self.uuid,
			}.into()
		).unwrap();
	}
}

impl Texture {
	pub fn new(render_engine: &Arc<RenderEngine>, data: &[u8], x_size: u32, y_size: u32) -> EngineFuture<Result<Arc<Texture>, ()>> {
		let (fut_send, fut_recv) = sync_channel(1);
		let (send, recv) = sync_channel(1);
		
		render_engine.command_channel.send(
			TextureCommand::CreateTexture { 
				send, 
				fut_send, 
				x_size, 
				y_size, 
				data: data.to_owned().into_boxed_slice(), 
				engine: render_engine.clone(),
			}.into()
		).unwrap();

		EngineFuture::new_single(recv)
			.with_wait_condition(fut_recv.into())
	}

	pub fn get_all(render_engine: &Arc<RenderEngine>) -> EngineFuture<Result<Box<[Arc<Texture>]>, ()>> {
		let (send, recv) = sync_channel(1);

		render_engine.command_channel.send(
			TextureCommand::GetTextures { 
				send: send,
			}.into()
		).unwrap();

		EngineFuture::new_single(recv)
	}
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
		render_engine::{RenderEngine, RenderEngineCreateInfo}, 
		texture::Texture,
	};

	const TEXTURE_DATA: [u8; 40000] = [0u8; 100 * 100 * 4];

	#[test]
	/// Ensure that `Texture::new()` and `Texture::drop()` are working as expected.
	fn new_texture() {
		let create_info = RenderEngineCreateInfo::new();
		let engine = RenderEngine::new(create_info).unwrap();

		let texture = Texture::new(&engine, &TEXTURE_DATA, 100, 100).unwrap().unwrap();

		drop(texture);

		let texture_list = Texture::get_all(&engine).unwrap().unwrap();

		assert!(texture_list.len() == 0);
	}

	#[test]
	/// Ensure that `Texture::get_all()` is working as expected.
	fn get_all() {
		let create_info = RenderEngineCreateInfo::new();
		let engine = RenderEngine::new(create_info).unwrap();

		let texture = Texture::new(&engine, &TEXTURE_DATA, 100, 100).unwrap().unwrap();

		let texture_list = Texture::get_all(&engine).unwrap().unwrap();

		assert!(texture_list.len() == 1);
		assert!(Arc::ptr_eq(&texture, &texture_list[0]));
	}
}
