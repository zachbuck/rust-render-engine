
use std::sync::Arc;

pub mod image_surface;

#[allow(private_bounds)]
pub trait Surface: SurfaceInfo {
	
}

pub(crate) trait SurfaceInfo {

}

impl<T> Surface for Arc<T> 
where T: Surface {

}

impl<T> SurfaceInfo for Arc<T>
where T: SurfaceInfo {
	
}
