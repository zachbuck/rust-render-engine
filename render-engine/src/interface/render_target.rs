
use std::sync::Arc;

pub mod render_object;

#[allow(private_bounds)]
pub trait RenderTarget: RenderTargetInfo {

}

pub(crate) trait RenderTargetInfo {

}

impl<T> RenderTarget for Arc<T> 
where T: RenderTarget {
	
}

impl<T> RenderTargetInfo for Arc<T>
where T: RenderTargetInfo {
	
}
