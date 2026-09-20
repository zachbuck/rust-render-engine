
use std::sync::Arc;

use crate::{
	engine_command::RenderInstruction, 
	render_object::RenderObject, 
	surface::Surface,
};

pub struct RenderInstructionBuffer {
	pub(crate) buffer: Box<[RenderInstruction]>
}

pub struct RenderInstructionBufferBuilder {
	buffer: Vec<RenderInstruction>,
}

impl RenderInstructionBufferBuilder {
	pub fn begin(surface: &dyn Surface) -> Self {
		let mut buffer = Vec::new();

		buffer.push(RenderInstruction::BeginRendering { 
			uuid: *surface.get_uuid(),
		});

		RenderInstructionBufferBuilder { buffer }
	}

	pub fn next_surface<'a>(&'a mut self, surface: &dyn Surface) -> &'a mut Self {
		self.buffer.push(RenderInstruction::EndRendering);
		self.buffer.push(RenderInstruction::BeginRendering { uuid: *surface.get_uuid() });

		self
	}

	pub fn render_object<'a>(&'a mut self, object: &Arc<RenderObject>) -> &'a mut Self {
		self.buffer.push(RenderInstruction::RenderObject { uuid: object.uuid });

		self
	}

	pub fn build(mut self) -> RenderInstructionBuffer {
		self.buffer.push(RenderInstruction::EndRendering);

		let buffer = self.buffer.into_boxed_slice();

		RenderInstructionBuffer { buffer }
	}
}
