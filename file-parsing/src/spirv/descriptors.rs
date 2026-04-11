
use std::{collections::HashMap, slice};

use crate::spirv::{Instruction, InstructionIterator, enumerants::{Decoration, StorageClass}};

#[derive(Debug)]
pub struct SpirvDescriptors {
	pub descriptors: Box<[Descriptor]>,
}

#[derive(Debug)]
pub struct Descriptor {
	name: String,
	set: u32,
	binding: u32,
	r#type: DescriptorType,
}

#[derive(Debug)]
pub enum DescriptorType {
	Struct(Box<[DescriptorType]>),

	Float,
	Vec2,
	Vec3,
	Vec4,
	Mat2,
	Mat3,
	Mat4,

	Double,
	DVec2,
	DVec3,
	DVec4,
	DMat2,
	DMat3,
	DMat4,

	Int,
	IVec2,
	IVec3,
	IVec4,
	IMat2,
	IMat3,
	IMat4,

	UInt,
	UVec2,
	UVec3,
	UVec4,
	UMat2,
	UMat3,
	UMat4,

	Bool,
	BVec2,
	BVec3,
	BVec4,
	BMat2,
	BMat3,
	BMat4,
}

impl SpirvDescriptors {
	pub(super) fn parse(instructions: InstructionIterator) -> Self {
		let mut decorations = HashMap::new();

		let mut type_definitions = HashMap::new();
		let mut constant_definitions = HashMap::new();
		let mut variable_definitions = HashMap::new();

		for (instruction_code, instruction) in instructions {
			match instruction_code {
				Instruction::OpName				=> { decorations.entry(instruction[0]).or_insert((None, None, None)).0 = unsafe { Some(String::from_utf8_unchecked(slice::from_raw_parts(instruction[1..].as_ptr() as *const u8, instruction[1..].len() * 4).into())) }; }
				Instruction::OpDecorate			=> {
					let decoration = decorations.entry(instruction[0]).or_insert((None, None, None));

					match Decoration::get_decoration(instruction[1]) {
						Decoration::Binding 		=> { decoration.2 = Some(instruction[2]); },
						Decoration::DescriptorSet 	=> { decoration.1 = Some(instruction[2]); },
						Decoration::Unknown 		=> (),
					}
				}

				Instruction::OpTypeVoid 		=> { type_definitions.insert(instruction[0], DescriptorTypeInternal::Void); },
				Instruction::OpTypeBool 		=> { type_definitions.insert(instruction[0], DescriptorTypeInternal::Bool); },
				Instruction::OpTypeInt 			=> { type_definitions.insert(instruction[0], DescriptorTypeInternal::Int(instruction[1], instruction[2] == 1)); },
				Instruction::OpTypeFloat 		=> { type_definitions.insert(instruction[0], DescriptorTypeInternal::Float(instruction[1])); },
				Instruction::OpTypeVector 		=> { type_definitions.insert(instruction[0], DescriptorTypeInternal::Vector(instruction[1], instruction[2])); },
				Instruction::OpTypeMatrix 		=> { type_definitions.insert(instruction[0], DescriptorTypeInternal::Matrix(instruction[1], instruction[2])); },
				Instruction::OpTypeArray		=> { type_definitions.insert(instruction[0], DescriptorTypeInternal::Array(instruction[1], instruction[2])); },
				Instruction::OpTypeRuntimeArray	=> { type_definitions.insert(instruction[0], DescriptorTypeInternal::RuntimeArray(instruction[1])); },
				Instruction::OpTypeStruct		=> { type_definitions.insert(instruction[0], DescriptorTypeInternal::Struct(instruction[1..].to_owned().into_boxed_slice())); },
				Instruction::OpTypePointer		=> { type_definitions.insert(instruction[0], DescriptorTypeInternal::Pointer(StorageClass::get_storage_class(instruction[1]), instruction[2])); }

				Instruction::OpConstantTrue		=> { constant_definitions.insert(instruction[0], DescriptorConstantInternal::True); },
				Instruction::OpConstantFalse	=> { constant_definitions.insert(instruction[0], DescriptorConstantInternal::False); },
				Instruction::OpConstant			=> { constant_definitions.insert(instruction[0], DescriptorConstantInternal::Typed(instruction[1], instruction[2..].to_owned().into_boxed_slice())); }
				
				Instruction::OpVariable			=> { variable_definitions.insert(instruction[1], (StorageClass::get_storage_class(instruction[2]), instruction[0])); }
				_ => ()
			}
		}

		let mut descriptors = Vec::new();

		for (variable_id, (_, pointer_type_id)) in variable_definitions.iter().filter(|(_, (storage, _))| *storage == StorageClass::Uniform) {
			let type_id;
			if let DescriptorTypeInternal::Pointer(_, i) = type_definitions.get(pointer_type_id).unwrap() {
				type_id = i;
			} else {
				panic!()
			}

			let descriptor = Descriptor {
				name: decorations.get(type_id).unwrap().0.as_ref().unwrap().trim_matches('\0').to_string(),
				set: decorations.get(variable_id).unwrap().1.unwrap(),
				binding: decorations.get(variable_id).unwrap().2.unwrap(),
				r#type: Self::get_descriptor_type(type_id, &type_definitions),
			};

			descriptors.push(descriptor);
		}

		SpirvDescriptors {
			descriptors: descriptors.into_boxed_slice(),
		}
	}

	fn get_descriptor_type(id: &u32, type_definitions: &HashMap<u32, DescriptorTypeInternal>) -> DescriptorType {
		let current = type_definitions.get(id).unwrap();
		match current {
			DescriptorTypeInternal::Float(32) 		=> DescriptorType::Float,
			DescriptorTypeInternal::Float(64) 		=> DescriptorType::Double,
			DescriptorTypeInternal::Int(32, false) 	=> DescriptorType::Int,
			DescriptorTypeInternal::Int(32, true) 	=> DescriptorType::UInt,
			DescriptorTypeInternal::Bool 			=> DescriptorType::Bool,
			DescriptorTypeInternal::Vector(type_id, 2) => {
				match Self::get_descriptor_type(type_id, type_definitions) {
					DescriptorType::Float 	=> DescriptorType:: Vec2,
					DescriptorType::Double 	=> DescriptorType::DVec2,
					DescriptorType::Int 	=> DescriptorType::IVec2,
					DescriptorType::UInt 	=> DescriptorType::UVec2,
					DescriptorType::Bool 	=> DescriptorType::BVec2,
					_ => unimplemented!()
				}
			},
			DescriptorTypeInternal::Vector(type_id, 3) => {
				match Self::get_descriptor_type(type_id, type_definitions) {
					DescriptorType::Float 	=> DescriptorType:: Vec3,
					DescriptorType::Double 	=> DescriptorType::DVec3,
					DescriptorType::Int 	=> DescriptorType::IVec3,
					DescriptorType::UInt 	=> DescriptorType::UVec3,
					DescriptorType::Bool 	=> DescriptorType::BVec3,
					_ => unimplemented!()
				}
			},
			DescriptorTypeInternal::Vector(type_id, 4) => {
				match Self::get_descriptor_type(type_id, type_definitions) {
					DescriptorType::Float 	=> DescriptorType:: Vec4,
					DescriptorType::Double 	=> DescriptorType::DVec4,
					DescriptorType::Int 	=> DescriptorType::IVec4,
					DescriptorType::UInt 	=> DescriptorType::UVec4,
					DescriptorType::Bool 	=> DescriptorType::BVec4,
					_ => unimplemented!()
				}
			},
			DescriptorTypeInternal::Matrix(type_id, 2) => {
				match Self::get_descriptor_type(type_id, type_definitions) {
					DescriptorType:: Vec2 	=> DescriptorType:: Mat2,
					DescriptorType::DVec2 	=> DescriptorType::DMat2,
					DescriptorType::IVec2 	=> DescriptorType::IMat2,
					DescriptorType::UVec2 	=> DescriptorType::UMat2,
					DescriptorType::BVec2 	=> DescriptorType::BMat2,
					_ => unimplemented!()
				}
			},
			DescriptorTypeInternal::Matrix(type_id, 3) => {
				match Self::get_descriptor_type(type_id, type_definitions) {
					DescriptorType:: Vec3 	=> DescriptorType:: Mat3,
					DescriptorType::DVec3 	=> DescriptorType::DMat3,
					DescriptorType::IVec3 	=> DescriptorType::IMat3,
					DescriptorType::UVec3 	=> DescriptorType::UMat3,
					DescriptorType::BVec3 	=> DescriptorType::BMat3,
					_ => unimplemented!()
				}
			},
			DescriptorTypeInternal::Matrix(type_id, 4) => {
				match Self::get_descriptor_type(type_id, type_definitions) {
					DescriptorType:: Vec4 	=> DescriptorType:: Mat4,
					DescriptorType::DVec4 	=> DescriptorType::DMat4,
					DescriptorType::IVec4 	=> DescriptorType::IMat4,
					DescriptorType::UVec4 	=> DescriptorType::UMat4,
					DescriptorType::BVec4 	=> DescriptorType::BMat4,
					_ => unimplemented!()
				}
			},
			DescriptorTypeInternal::Struct(types) => {
				DescriptorType::Struct(types.iter().map(|i| Self::get_descriptor_type(i, type_definitions)).collect::<Vec<_>>().into_boxed_slice())
			}
			_ => unimplemented!()
		}
	}
}

#[derive(Debug)]
enum DescriptorTypeInternal {
	Void,
	Bool,
	Int(u32, bool), 			// Int(width, signed) 
	Float(u32),					// Float(width)
	Vector(u32, u32),			// Vector(type_id, count)
	Matrix(u32, u32),			// Matrix(type_id, count)
	Array(u32, u32),			// Array(type_id, length_id)
	RuntimeArray(u32),			// RuntimeArray(type_id)
	Struct(Box<[u32]>), 		// Struct(Box<[component_id]>)
	Pointer(StorageClass, u32),	// Pointer(storage_class, type_id)
}

#[derive(Debug)]
enum DescriptorConstantInternal {
	True,
	False,
	Typed(u32, Box<[u32]>),	// Typed(type_id, data)
}
