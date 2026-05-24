
use crate::interface::{
	RenderEngine,
	engine_future::{
		EngineFuture,
		immediate_engine_future::ImmediateEngineFuture,
	},
	render_target::RenderTarget,
	surface::Surface,
};

pub struct InstructionBufferBuilder {

}

pub struct InstructionBuffer {

}

impl InstructionBufferBuilder {
	pub fn begin() -> Self {
		todo!()
	}

	pub fn bind_surface<'a>(&'a mut self, surface: &dyn Surface) -> &'a mut Self {
		todo!()
	}

	pub fn draw<'a>(&'a mut self, render_target: &dyn RenderTarget) -> &'a mut Self {
		todo!()
	}

	pub fn build(self) -> InstructionBuffer {
		todo!()
	}
}

impl InstructionBuffer {
	pub fn submit(&self) -> impl EngineFuture<Result<(), ()>> {
		todo!() as ImmediateEngineFuture<_>
	}
}
