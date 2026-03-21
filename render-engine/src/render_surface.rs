
use std::any::Any;

pub mod image_surface;

pub(crate) trait RenderSurface: Any {
	fn as_any(&self) -> &dyn Any;
	fn as_mut_any(&mut self) -> &mut dyn Any;
}
