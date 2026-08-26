
use std::sync::Arc;

use uuid::Uuid;

pub mod window_surface;

#[expect(private_bounds)]
pub trait Surface: SurfaceInfo {}

pub(crate) trait SurfaceInfo {
	fn get_uuid(&self) -> &Uuid;
}

impl<T> Surface for Arc<T> where T: Surface {}

impl<T> SurfaceInfo for Arc<T> 
where T: SurfaceInfo {
	fn get_uuid(&self) -> &Uuid { self.as_ref().get_uuid() }
}

#[derive(Debug)]
pub struct RenderPassCreateInfo {

}

impl Default for RenderPassCreateInfo {
	fn default() -> Self {
		RenderPassCreateInfo {

		}
	}
}
