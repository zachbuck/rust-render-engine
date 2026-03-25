
use std::sync::{Arc, Weak};

use vulkano::image::view::ImageView;

use crate::texture::Texture;

pub(crate) struct TextureInternal {
	pub(crate) reference: Weak<Texture>,
	#[expect(dead_code)]
	pub(crate) image: Arc<ImageView>,
}
