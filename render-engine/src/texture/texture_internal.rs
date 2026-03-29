
use std::{collections::HashMap, sync::{Arc, Weak}};

use uuid::Uuid;
use vulkano::image::{
	sampler::Sampler, 
	view::ImageView,
};

use crate::{
	render_engine::render_thread::RenderThread, 
	texture::Texture,
};

#[derive(Debug)]
pub(crate) struct TextureInternal {
	pub(crate) reference: Weak<Texture>,
	pub(crate) image: Arc<ImageView>,
	pub(crate) sampler: Arc<Sampler>
}

impl RenderThread {
	#[inline]
	#[expect(dead_code)]
	pub(crate) fn get_texture<'a>(textures: &'a HashMap<Uuid, TextureInternal>, uuid: &Uuid) -> Option<&'a TextureInternal> {
		textures.get(uuid)
	}

	#[inline]
	#[expect(dead_code)]
	pub(crate) fn get_mut_texture<'a>(textures: &'a mut HashMap<Uuid, TextureInternal>, uuid: &Uuid) -> Option<&'a mut TextureInternal> {
		textures.get_mut(uuid)
	}
}
