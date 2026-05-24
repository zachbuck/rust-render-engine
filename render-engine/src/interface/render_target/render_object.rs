
use std::sync::Arc;

use crate::interface::{
	RenderEngine,
	engine_future::{
		EngineFuture,
		immediate_engine_future::ImmediateEngineFuture,
	},
	mesh_data::MeshData, 
	pipeline::GraphicsPipeline,
	render_target::{RenderTarget, RenderTargetInfo},
};

pub struct RenderObject<V, I> {
	pub mesh: Arc<MeshData<V, I>>,
	pub pipeline: Arc<GraphicsPipeline>,
}

impl<V, I> RenderObject<V, I> {
	pub fn new(render_engine: &RenderEngine, mesh: &Arc<MeshData<V, I>>, pipeline: &Arc<GraphicsPipeline>) -> impl EngineFuture<Result<Arc<Self>, ()>> {
		todo!() as ImmediateEngineFuture<_>
	}
}

impl<V, I> Drop for RenderObject<V, I> {
	fn drop(&mut self) {
		todo!()
	}
}

impl<V, I> RenderTarget for RenderObject<V, I> {

}

impl<V, I> RenderTargetInfo for RenderObject<V, I> {

}
