use std::sync::Arc;

use crate::{render_engine::{RenderEngine, RenderEngineCreateInfo}, texture::Texture};


#[test]
/// Ensure `Texture::new()` and `Texture::drop()` are working as expected.
fn new_texture() {
	let create_info = RenderEngineCreateInfo::new();
	let engine = RenderEngine::new(create_info).unwrap();

	let data = [0u8; 100 * 100 * 4];
	
	let texture = Texture::new(engine.clone(), &data, 100, 100).unwrap().unwrap();
	drop(texture);

	let textures = Texture::get_all(engine.clone()).unwrap().unwrap();
	assert!(textures.len() == 0);
}

#[test]
/// Ensure `Texture::get_all()` is working as epected.
fn get_texture() {
	let create_info = RenderEngineCreateInfo::new();
	let engine = RenderEngine::new(create_info).unwrap();

	let data = [0u8; 100 * 100 * 4];

	let texture = Texture::new(engine.clone(), &data, 100, 100).unwrap().unwrap();

	let textures = Texture::get_all(engine.clone()).unwrap().unwrap();

	assert!(textures.len() == 1);
	assert!(Arc::ptr_eq(&texture, &textures[0]));
}


