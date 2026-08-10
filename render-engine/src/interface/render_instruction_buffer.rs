
use crate::{
	engine_command::RenderInstruction, 
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

	pub fn build(mut self) -> RenderInstructionBuffer {
		self.buffer.push(RenderInstruction::EndRendering);

		let buffer = self.buffer.into_boxed_slice();

		RenderInstructionBuffer { buffer }
	}
}
