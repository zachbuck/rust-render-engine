
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
	pub fn get_shader_stage(binary: &[u32]) -> ShaderStage {
		let mut iter = InstructionIterator::new(binary);
		let instruction = iter.next_instruction();
		while !instruction.is_none() {
			let (instruction, data) = instruction.unwrap();
		}
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

	fn next_instruction(&mut self) -> Option<(Instruction, &[u32])> {
		if self.current_word == self.binary.len() { return None; }

		let first = self.binary[self.current_word];

		let word_count = (0xFF00 & first >> 16) as u16;
		let instruction = (0x00FF & first) as u16;

		let out = Some((Instruction::from(instruction), &self.binary[self.current_word + 1..self.current_word + (word_count as usize)]));

		self.current_word += word_count as usize;

		return out;
	}
}
