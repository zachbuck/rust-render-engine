
use crate::spirv::{
	compiler::ShaderStage, data_type::DataType, enumerations::{ExecutionModel, Instruction, StorageClass},
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

			stage = Some(ExecutionModel::from(data[1]));
			break;
		}

		stage.unwrap().into()
	}

	pub fn get_input_output_layout(binary: &[u32]) -> () {
		let bound = binary[3];

		#[derive(Debug)]
		#[derive(Clone, Copy)]
		enum Constant {
			Float(f32),
			Double(f64),
			Int(i32),
			UInt(u32),
		}

		#[derive(Debug)]
		#[derive(Clone)]
		enum ID {
			Error,
			DataType(DataType),
			Pointer(DataType),
			Constant(Constant),
			Variable(StorageClass),
		}

		let mut id_values = (0..bound).map(|_| None).collect::<Box<[_]>>();
		let mut iter = InstructionIterator::new(binary);
		while !(iter.is_at_end()) {
			let (instruction, data) = iter.next_instruction().unwrap();
			match instruction {
				Instruction::OpTypeInt 		=> id_values[data[1] as usize] = Some(match (data[2], data[3] != 0) {
												(32, true) 	=> ID::DataType(DataType::Int),
												(32, false)	=> ID::DataType(DataType::UInt),
												_ 			=> ID::Error,
											}),
				Instruction::OpTypeFloat	=> id_values[data[1] as usize] = Some(match data[2] {
												32 	=> ID::DataType(DataType::Float),
												64 	=> ID::DataType(DataType::Double),
												_	=> ID::Error,
											}),
				Instruction::OpTypeVector	=> id_values[data[1] as usize] = Some(match (id_values[data[2] as usize].clone(), data[3]) {
												(Some(ID::DataType(DataType::Float)), 2) 	=> ID::DataType(DataType::Vec2),
												(Some(ID::DataType(DataType::Float)), 3) 	=> ID::DataType(DataType::Vec3),
												(Some(ID::DataType(DataType::Float)), 4) 	=> ID::DataType(DataType::Vec4),

												(Some(ID::DataType(DataType::Double)), 2) 	=> ID::DataType(DataType::DVec2),
												(Some(ID::DataType(DataType::Double)), 3) 	=> ID::DataType(DataType::DVec3),
												(Some(ID::DataType(DataType::Double)), 4) 	=> ID::DataType(DataType::DVec4),

												(Some(ID::DataType(DataType::Int)), 2) 		=> ID::DataType(DataType::IVec2),
												(Some(ID::DataType(DataType::Int)), 3) 		=> ID::DataType(DataType::IVec3),
												(Some(ID::DataType(DataType::Int)), 4) 		=> ID::DataType(DataType::IVec4),
												
												(Some(ID::DataType(DataType::UInt)), 2) 	=> ID::DataType(DataType::UVec2),
												(Some(ID::DataType(DataType::UInt)), 3) 	=> ID::DataType(DataType::UVec3),
												(Some(ID::DataType(DataType::UInt)), 4) 	=> ID::DataType(DataType::UVec4),
												
												_ 											=> ID::Error,
											}),
				Instruction::OpTypeMatrix	=> id_values[data[1] as usize] = Some(match (id_values[data[2] as usize].clone(), data[3]) {
												(Some(ID::DataType(DataType::Vec2)), 2) 	=> ID::DataType(DataType::Mat2),
												(Some(ID::DataType(DataType::Vec3)), 3) 	=> ID::DataType(DataType::Mat3),
												(Some(ID::DataType(DataType::Vec4)), 4) 	=> ID::DataType(DataType::Mat4),

												(Some(ID::DataType(DataType::DVec2)), 2) 	=> ID::DataType(DataType::DMat2),
												(Some(ID::DataType(DataType::DVec3)), 3) 	=> ID::DataType(DataType::DMat3),
												(Some(ID::DataType(DataType::DVec4)), 4) 	=> ID::DataType(DataType::DMat4),
												
												(Some(ID::DataType(DataType::IVec2)), 2) 	=> ID::DataType(DataType::IMat2),
												(Some(ID::DataType(DataType::IVec3)), 3) 	=> ID::DataType(DataType::IMat3),
												(Some(ID::DataType(DataType::IVec4)), 4) 	=> ID::DataType(DataType::IMat4),
												
												(Some(ID::DataType(DataType::UVec2)), 2) 	=> ID::DataType(DataType::UMat2),
												(Some(ID::DataType(DataType::UVec3)), 3) 	=> ID::DataType(DataType::UMat3),
												(Some(ID::DataType(DataType::UVec4)), 4) 	=> ID::DataType(DataType::UMat4),
												
												_ => ID::Error,
											}),
				Instruction::OpTypeStruct	=> id_values[data[1] as usize] = Some({
												let mut data_types = data[2..].iter()
													.map(|id| {
														if let Some(ID::DataType(data_type)) = &id_values[*id as usize] {
															Some(data_type.clone())
														} else {
															None
														}
													}).collect::<Box<[_]>>();
												
												if data_types.iter().find(|dt| dt.is_none()).is_some() {
													println!("{:?}", &data[2..]);
													ID::Error
												} else {
													ID::DataType(DataType::Struct { elements: data_types.iter_mut().map(|dt| dt.take().unwrap()).collect() })
												}
											}),
				_ 							=> (),
			}
		}

		for (id, value) in id_values.iter().enumerate() {
			println!("{:2?}: {:?}", id, value);
		}
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

		let out = Some((Instruction::from(instruction), &self.binary[self.current_word..self.current_word + (word_count as usize)]));

		self.current_word += word_count as usize;

		return out;
	}

	fn is_at_end(&self) -> bool { self.current_word == self.binary.len() }
}
