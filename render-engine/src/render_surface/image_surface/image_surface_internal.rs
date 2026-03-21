
use std::sync::Arc;

use vulkano::image::view::ImageView;

use crate::{
	render_engine::render_thread::RenderThread, 
	render_surface::{
		RenderSurface, 
		image_surface::ImageSurface
	}
};

pub(crate) struct ImageSurfaceInternal {
	pub(crate) image: Arc<ImageView>,
}

impl RenderSurface for ImageSurfaceInternal {
	fn as_any(&self) -> &dyn std::any::Any { self }

	fn as_mut_any(&mut self) -> &mut dyn std::any::Any { self }
}

impl RenderThread {
	#[inline]
	#[expect(dead_code)]
	pub(crate) fn get_image_surface(&self, reference: Arc<ImageSurface>) -> Option<&ImageSurfaceInternal> {
		self.render_surfaces.get(&reference.uuid)?.as_any().downcast_ref()
	}

	#[inline]
	#[expect(dead_code)]
	pub(crate) fn get_mut_image_surface(&mut self, reference: Arc<ImageSurface>) -> Option<&mut ImageSurfaceInternal> {
		self.render_surfaces.get_mut(&reference.uuid)?.as_mut_any().downcast_mut()
	}
}
