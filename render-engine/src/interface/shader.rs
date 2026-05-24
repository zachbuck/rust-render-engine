
use std::sync::Arc;

use crate::interface::{
	RenderEngine,
	engine_future::{
		EngineFuture,
		immediate_engine_future::ImmediateEngineFuture,
	},
};

pub struct Shader {
	pub stage: ShaderStage,
}

#[repr(u32)]
pub enum ShaderStage {
	Vertex = 	0x0001,
	Fragment = 	0x0002,
}

impl Shader {
	pub fn from_spirv(render_engine: &RenderEngine, binary: &[u32]) -> impl EngineFuture<Result<Arc<Self>, ()>> {
		todo!() as ImmediateEngineFuture<_>
	}
}

impl Drop for Shader {
	fn drop(&mut self) {
		todo!()
	}
}
