
use std::{
	marker::PhantomData, 
	sync::Arc,
};

use crate::interface::{
	RenderEngine,
	engine_future::{
		EngineFuture,
		immediate_engine_future::ImmediateEngineFuture,
	},
};

pub struct MeshData<V, I> {
	vertex_format: PhantomData<V>,
	index_length: PhantomData<I>,
}

impl<V, I> MeshData<V, I> {
	pub fn new(render_engine: &RenderEngine, vertices: Box<[V]>, indices: Box<[I]>) -> impl EngineFuture<Result<Arc<Self>, ()>> {
		todo!() as ImmediateEngineFuture<_>
	}
}

impl<V, I> Drop for MeshData<V, I> {
	fn drop(&mut self) {
		todo!()
	}
}
