
use std::sync::Arc;

use crate::interface::{
	RenderEngine,
	engine_future::{
		EngineFuture,
		immediate_engine_future::ImmediateEngineFuture,
	}, 
	shader::{Shader, ShaderStage},
};

pub struct GraphicsPipeline {
	pub stages: ShaderStages,
	pub shaders: Box<[Arc<Shader>]>,
}

pub struct ShaderStages {
	stages: u32,
}

impl GraphicsPipeline {
	pub fn new(render_engine: &RenderEngine, shaders: &[&Arc<Shader>]) -> impl EngineFuture<Result<Arc<Self>, ()>> {
		todo!() as ImmediateEngineFuture<_>
	}
}

impl Drop for GraphicsPipeline {
	fn drop(&mut self) {
		todo!()
	}
}

impl ShaderStages {
	pub fn empty() -> Self { ShaderStages { stages: 0x0000 } }
	pub fn add(&mut self, stage: ShaderStage) { self.stages &= stage as u32; }
	pub fn has_stage(&self, stage: ShaderStage) -> bool { self.stages & stage as u32 != 0 }
}
