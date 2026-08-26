
use crate::spirv::{
	compiler::ShaderStage, 
	enumerations::Instruction,
};

pub struct Interpreter {}

struct InstructionIterator<'a> {
	current_word: 	usize,
	binary: 		&'a[u32]
}

impl Interpreter {
	fn get_shader_stage(binary: &[u32]) -> ShaderStage {
		todo!()
	}
}

impl<'a> InstructionIterator<'a> {
	fn new(binary: &'a[u32]) -> Self {
		InstructionIterator {
			current_word: 5,
			binary: binary,
		}
	}

	fn next_word(&mut self) -> Option<(Instruction, &[u32])> {
		let first = self.binary[self.current_word];

		let word_count = (0xFF00 & first >> 16) as u16;
		let instruction = (0x00FF & first) as u16;

		self.current_word += word_count as usize;

		todo!()
	}
}
