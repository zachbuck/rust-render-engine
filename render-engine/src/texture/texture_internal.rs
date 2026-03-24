
use std::sync::{Arc, Weak};

use vulkano::image::view::ImageView;

use crate::texture::Texture;

pub(crate) struct TextureInternal {
	pub(crate) reference: Weak<Texture>,
	pub(crate) image: Arc<ImageView>,
}
