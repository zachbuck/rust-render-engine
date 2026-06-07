
use std::sync::Arc;

use uuid::Uuid;

use crate::interface::pipeline::GraphicsPipeline;

pub mod render_object;

#[allow(private_bounds)]
pub trait RenderTarget: RenderTargetInfo {

}

pub(crate) trait RenderTargetInfo {
	fn get_uuid(&self) -> &Uuid;
	
	fn get_pipeline(&self) -> &Arc<GraphicsPipeline>;
}

impl<T> RenderTarget for Arc<T> 
where T: RenderTarget {
	
}

impl<T> RenderTargetInfo for Arc<T>
where T: RenderTargetInfo {
	fn get_uuid(&self) -> &Uuid { self.as_ref().get_uuid() }
	fn get_pipeline(&self) -> &Arc<GraphicsPipeline> { self.as_ref().get_pipeline() }
}
