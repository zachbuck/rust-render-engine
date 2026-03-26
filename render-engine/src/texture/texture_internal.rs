
use std::sync::{Arc, Weak};

use vulkano::image::{
	sampler::Sampler, 
	view::ImageView,
};

use crate::texture::Texture;

#[derive(Debug)]
pub(crate) struct TextureInternal {
	pub(crate) reference: Weak<Texture>,
	pub(crate) image: Arc<ImageView>,
	pub(crate) sampler: Arc<Sampler>
}
