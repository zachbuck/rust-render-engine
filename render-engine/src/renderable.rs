
use std::any::Any;

pub mod render_object;

pub(crate) trait Renderable: Any {
	fn as_any(&self) -> &dyn Any;
	fn as_mut_any(&mut self) -> &mut dyn Any;
}
