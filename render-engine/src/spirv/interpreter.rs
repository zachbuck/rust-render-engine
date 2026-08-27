
use crate::spirv::{
	compiler::ShaderStage, enumerations::{ExecutionModel, Instruction, StorageClass},
};

pub struct Interpreter {}

struct InstructionIterator<'a> {
	current_word: 	usize,
	binary: 		&'a[u32]
}

impl Interpreter {
	pub fn get_shader_stage(binary: &[u32]) -> ShaderStage {
		let mut iter = InstructionIterator::new(binary);
		let mut stage = None;

		while !iter.is_at_end() {
			let (instruction, data) = iter.next_instruction().unwrap();
			if !(instruction == Instruction::OpEntryPoint) { continue; }

			stage = Some(ExecutionModel::from(data[0]));
			break;
		}

		stage.unwrap().into()
	}

	pub fn get_input_output_layout(binary: &[u32]) -> () {
		let bound = binary[3];

		#[derive(Debug)]
		enum IdInfo {
			Variable{ type_id: u32, storage_class: StorageClass },
		}

		let mut ids = (0..bound).map(|_| None).collect::<Vec<_>>();

		let mut iter = InstructionIterator::new(binary);
		while !iter.is_at_end() {
			let (instruction, data) = iter.next_instruction().unwrap();
			match instruction {
				Instruction::OpVariable => { ids[data[1] as usize] = Some(IdInfo::Variable { type_id: data[0], storage_class: data[2].into() } ) },
				_ => ()
			}
		}

		println!("{:?}", ids);
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

		let word_count = (first >> 16) as u16;
		let instruction = (0x00FF & first) as u16;

		let out = Some((Instruction::from(instruction), &self.binary[self.current_word + 1..self.current_word + (word_count as usize)]));

		self.current_word += word_count as usize;

		return out;
	}

	fn is_at_end(&self) -> bool { self.current_word == self.binary.len() }
}
