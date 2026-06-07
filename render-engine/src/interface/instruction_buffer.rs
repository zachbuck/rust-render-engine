
use std::collections::LinkedList;

use uuid::Uuid;

use crate::interface::{
	RenderEngine,
	engine_command::{InstructionBufferCommand, EngineCommand},
	engine_future::{
		EngineFuture,
		channel_engine_future::ChannelEngineFuture
	},
	render_target::RenderTarget,
	surface::Surface,
};

pub struct InstructionBufferBuilder {
	current_surface: Option<Uuid>,
	instructions: LinkedList<Instruction>,
}

#[derive(Clone)]
pub struct InstructionBuffer {
	pub(crate) instructions: Box<[Instruction]>
}

#[derive(Clone, Copy)]
pub(crate) enum Instruction {
	BeginSurface(Uuid),
	DrawRenderTarget(Uuid),
	EndSurface(Uuid),
}

impl InstructionBufferBuilder {
	pub fn new() -> Self {
		InstructionBufferBuilder {
			current_surface: None,
			instructions: LinkedList::new(),
		}
	}

	pub fn bind_surface<'a>(&'a mut self, surface: &dyn Surface) -> Result<&'a mut Self, ()> {
		if self.current_surface.is_some() {
			let instruction = Instruction::EndSurface(self.current_surface.unwrap());
			self.instructions.push_back(instruction);
		}
		
		let uuid = *surface.get_uuid();
		self.current_surface = Some(uuid);
		let instruction = Instruction::BeginSurface(uuid);
		self.instructions.push_back(instruction);

		Ok(self)
	}

	pub fn draw<'a>(&'a mut self, render_target: &dyn RenderTarget) -> Result<&'a mut Self, ()> {
		if self.current_surface.is_none_or(|uuid| uuid != *render_target.get_pipeline().surface.get_uuid()) { return Err(()) }

		let instruction = Instruction::DrawRenderTarget(*render_target.get_uuid());
		self.instructions.push_back(instruction);

		Ok(self)
	}

	pub fn build(mut self) -> InstructionBuffer {
		if self.current_surface.is_some() {
			let instruction = Instruction::EndSurface(self.current_surface.unwrap());
			self.instructions.push_back(instruction);
		}

		let instructions = self.instructions.into_iter().collect::<Vec<_>>().into_boxed_slice();
		InstructionBuffer {
			instructions,
		}
	}
}

impl InstructionBuffer {
	pub fn submit(self, render_engine: &RenderEngine) -> impl EngineFuture<Result<(), ()>> {
		let (future, response) = ChannelEngineFuture::new();

		let command = InstructionBufferCommand::OneTimeSubmit { instructions: self, response };
		let command = EngineCommand::InstructionBufferCommand(Box::new(command));
		let _ = render_engine.command_channel.send(command);

		return future
	}
}
