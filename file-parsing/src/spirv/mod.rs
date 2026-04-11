
use crate::spirv::{descriptors::SpirvDescriptors, enumerants::Instruction};

pub mod descriptors;
mod enumerants;

pub struct SpirvParser {
	pub version: (u8, u8),
	descriptors: Option<SpirvDescriptors>,
}

pub struct SpirvParserFlags {
	pub descriptors: bool,
}

impl SpirvParser {
	pub fn parse(binary: &[u32], flags: SpirvParserFlags) -> Self {
		let iterator = InstructionIterator::new(&binary[5..]);

		let descriptors;
		if flags.descriptors { descriptors = Some(SpirvDescriptors::parse(iterator.clone())); } else { descriptors = None; }

		let version = (
			(binary[1] & 0x00FF0000 >> 16) as u8,
			(binary[1] & 0x0000FF00 >> 08) as u8,
		);

		SpirvParser { 
			version,
			descriptors,
		}
	}

	pub fn get_descriptors(&self) -> &SpirvDescriptors {
		self.descriptors.as_ref().unwrap()
	}
}

struct InstructionIterator<'a> {
	position: usize,
	binary: &'a[u32],
}

impl<'a> InstructionIterator<'a> {
	fn new(binary: &'a [u32]) -> Self {
		InstructionIterator { 
			position: 0, 
			binary: binary, 
		}
	}
}

impl<'a> Iterator for InstructionIterator<'a> {
	type Item = (Instruction, &'a [u32]);
	fn next(&mut self) -> Option<Self::Item> {
		if self.position == self.binary.len() { return None }

		let first_word = self.binary[self.position];
		let instruction_length = ((0xFFFF0000 & first_word) >> 16) as usize;
		let instruction_code = Instruction::get_instruction((0x0000FFFF & first_word) as u16);

		let full_instruction = &self.binary[(self.position + 1)..(self.position + instruction_length)];

		self.position += instruction_length;

		return Some((instruction_code, full_instruction));
	}
}

impl<'a> Clone for InstructionIterator<'a> {
	fn clone(&self) -> Self {
		InstructionIterator {
			position: self.position,
			binary: self.binary,
		}
	}
}
