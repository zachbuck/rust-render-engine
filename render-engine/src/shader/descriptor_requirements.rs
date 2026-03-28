
use std::{
	collections::{BTreeMap, HashMap}, 
	sync::Arc
};

use vulkano::{
	descriptor_set::layout::{DescriptorBindingFlags, DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutCreateFlags, DescriptorSetLayoutCreateInfo}, 
	device::Device, 
	shader::ShaderStages,
};

use crate::{macros::error_map, shader::ShaderType};

#[derive(Debug)]
#[derive(Clone)]
/// each element of descriptors is in the format 
/// .0.0: 	the set of the descriptor
/// .0.1: 	the binding of the descriptor
/// .1:		the type of the descriptor
/// .2:		the stages the descriptor is used in
pub struct DescriptorRequirements {
	pub(crate) descriptors: Box<[((u32, u32), DescriptorType, ShaderStages)]>
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq, Eq)]
pub enum DescriptorType {
	CombinedImageSampler,
	UniformBuffer(Box<[UniformBufferElement]>),
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq, Eq)]
pub enum UniformBufferElement {
	Float,
	Vec2,
	Vec3,
	Vec4,
	Mat2,
	Mat3,
	Mat4,
}

#[derive(Debug)]
#[derive(Default)]
struct DecorationSet {
	set: Option<u32>,
	binding: Option<u32>,
}

#[derive(Debug)]
#[expect(unused)]
enum SpirvType {
	Void,
	Bool,
	Int(u32, bool),
	Float(u32),
	Vector(u32, u32),
	Matrix(u32, u32),
	Image,
	Sampler,
	SampledImage(u32),
	Array(u32, u32),
	RuntimeArray(u32),
	Struct(Box<[u32]>),
	Pointer(u32),
	Function,
}

#[derive(Debug)]
#[expect(unused)]
enum Constant {
	True,
	False,
	Type(u32, Box<[u32]>),
}

#[derive(Debug)]
struct Variable {
	var_type: u32,
	storage_class: StorageClass,
}

impl DescriptorRequirements {
	pub(crate) fn from_binary(binary: &[u32], shader_stage: ShaderType) -> Self {
		let mut decorations: HashMap<u32, DecorationSet> = HashMap::new();
		let mut types: HashMap<u32, SpirvType> = HashMap::new();
		let mut constants: HashMap<u32, Constant> = HashMap::new();
		let mut variables: HashMap<u32, Variable> = HashMap::new();

		let mut i = 5;
		while i < binary.len() {
			let word_count = ((0xFFFF0000 & binary[i]) >> 16) as u16;
			let op_code = (0x0000FFFF & binary[i]) as u16;
			let instruction = &binary[i..i+(word_count as usize)];

			match OpCode::get_op_code(op_code) {
				OpCode::OpNop => (),

				OpCode::OpDecorate => { decorations.entry(instruction[1]).or_default().apply_decoration(instruction); },

				OpCode::OpTypeVoid => { types.insert(instruction[1], SpirvType::Void); },
				OpCode::OpTypeBool => { types.insert(instruction[1], SpirvType::Bool); },
				OpCode::OpTypeInt => { types.insert(instruction[1], SpirvType::Int(instruction[2], instruction[3] == 1)); },
				OpCode::OpTypeFloat => { types.insert(instruction[1], SpirvType::Float(instruction[2])); },
				OpCode::OpTypeVector => { types.insert(instruction[1], SpirvType::Vector(instruction[2], instruction[3])); },
				OpCode::OpTypeMatrix => { types.insert(instruction[1], SpirvType::Matrix(instruction[2], instruction[3])); },
				OpCode::OpTypeImage => { types.insert(instruction[1], SpirvType::Image); },
				OpCode::OpTypeSampler => { types.insert(instruction[1], SpirvType::Sampler); },
				OpCode::OpTypeSampledImage => { types.insert(instruction[1], SpirvType::SampledImage(instruction[2])); },
				OpCode::OpTypeArray => { types.insert(instruction[1], SpirvType::Array(instruction[2], instruction[3])); },
				OpCode::OpTypeRuntimeArray => { types.insert(instruction[1], SpirvType::RuntimeArray(instruction[2])); },
				OpCode::OpTypeStruct => { types.insert(instruction[1], SpirvType::Struct(instruction[2..].to_owned().into_boxed_slice())); },
				OpCode::OpTypePointer => { types.insert(instruction[1], SpirvType::Pointer(instruction[3])); },
				OpCode::OpTypeFunction => { types.insert(instruction[1], SpirvType::Function); },

				OpCode::OpConstantTrue => { constants.insert(instruction[2], Constant::True); },
				OpCode::OpConstantFalse => { constants.insert(instruction[2], Constant::False); },
				OpCode::OpConstant => { constants.insert(instruction[2], Constant::Type(instruction[1], instruction[3..].to_vec().into_boxed_slice())); },

				OpCode::OpVariable => { variables.insert(instruction[2], Variable { var_type: instruction[1], storage_class: StorageClass::from_code(instruction[3]) }); },
			}

			i += word_count as usize;
		}

		let mut descriptors = Vec::new();

		for (variable_id, variable) in variables {
			if variable.storage_class != StorageClass::Uniform && variable.storage_class != StorageClass::UniformConstant { continue; }

			let decorator = decorations.get(&variable_id).unwrap();
			let set = decorator.set.unwrap();
			let binding = decorator.binding.unwrap();

			let descriptor_type;
			let variable_pointer_type = types.get(&variable.var_type).unwrap();
			if let SpirvType::Pointer(variable_type_id) = variable_pointer_type {
				let variable_type = types.get(variable_type_id).unwrap();
				if let SpirvType::Struct(elements) = variable_type {
					let mut uniform_buffer_elements = Vec::with_capacity(elements.len());

					for element in elements {
						uniform_buffer_elements.push(Self::get_struct_format(element, &types));
					}

					descriptor_type = DescriptorType::UniformBuffer(uniform_buffer_elements.into_boxed_slice());
				} else if let SpirvType::SampledImage(_) = variable_type {
					descriptor_type = DescriptorType::CombinedImageSampler;
				} else {
					panic!()
				}
			} else {
				panic!();
			}

			descriptors.push(((set, binding), descriptor_type, shader_stage.into()));
		}

		descriptors.sort_by_key(|((set, binding), _, _)| ((*set as u64) << 16) + (*binding as u64));

		DescriptorRequirements {
			descriptors: descriptors.into_boxed_slice(),
		}
	}

	pub(crate) fn test_compatibility(requirements: &[DescriptorRequirements]) -> bool {
		let mut descriptor_set = HashMap::new();

		for requirement in requirements {
			for (binding, descriptor_type, _) in &requirement.descriptors {
				if descriptor_set.contains_key(&binding) {
					return *descriptor_set.get(&binding).unwrap() == descriptor_type;
				} else {
					descriptor_set.insert(binding, descriptor_type);
				}
			}
		}

		return true;
	}

	pub(crate) fn combine(requirements: &[DescriptorRequirements]) -> Self {
		let mut descriptor_set = HashMap::new();

		for requirement in requirements {
			for (binding, descriptor_type, shader_stages) in &requirement.descriptors {
				let descriptor = descriptor_set
					.entry(binding)
					.or_insert((descriptor_type, ShaderStages::empty()));

				descriptor.1 = descriptor.1.union(*shader_stages);
			}
		}

		let mut descriptors = Vec::with_capacity(descriptor_set.len());
		for (binding, (descriptor_type, shader_stages)) in descriptor_set {
			descriptors.push((*binding, descriptor_type.clone(), shader_stages));
		}

		descriptors.sort_by_key(|((set, binding), _, _)| ((*set as u64) << 16) + (*binding as u64));

		DescriptorRequirements {
			descriptors: descriptors.into_boxed_slice(),
		}
	}

	pub(crate) fn get_descriptor_layouts(&self, device: &Arc<Device>) -> Result<HashMap<u32, Arc<DescriptorSetLayout>>, ()> {
		let sets = self.descriptors.chunk_by(|((set1, _), _, _), ((set2, _), _, _)| set1 == set2);
		let mut out = HashMap::new();

		for set in sets {
			let mut bindings = BTreeMap::new();

			for ((_, binding), descriptor_type, shader_stages) in set {
				let descriptor_set_layout_binding = 
				match descriptor_type {
						DescriptorType::CombinedImageSampler => DescriptorSetLayoutBinding {
							binding_flags: DescriptorBindingFlags::empty(),
							descriptor_count: 1,
							stages: *shader_stages,
							immutable_samplers: Vec::new(),
							..DescriptorSetLayoutBinding::descriptor_type(vulkano::descriptor_set::layout::DescriptorType::CombinedImageSampler)
						},
						DescriptorType::UniformBuffer(_) => DescriptorSetLayoutBinding { 
							binding_flags: DescriptorBindingFlags::empty(), 
							descriptor_count: 1, 
							stages: *shader_stages, 
							immutable_samplers: Vec::new(), 
							..DescriptorSetLayoutBinding::descriptor_type(vulkano::descriptor_set::layout::DescriptorType::UniformBuffer)
						},
					};
				
				bindings.insert(*binding, descriptor_set_layout_binding);
			}

			let set_layout = DescriptorSetLayout::new(
				device.clone(), 
				DescriptorSetLayoutCreateInfo {
					flags: DescriptorSetLayoutCreateFlags::empty(),
					bindings: bindings,
					..Default::default()
				}
			).map_err(error_map!())?;

			out.insert(set[0].0.0, set_layout);
		}

		out.shrink_to_fit();
		return Ok(out);
	}

	fn get_struct_format(id: &u32, types: &HashMap<u32, SpirvType>) -> UniformBufferElement {
		match types.get(id).unwrap() {
			SpirvType::Bool => todo!("Boolean uniforms are not currently supported"),
			SpirvType::Int(_, _) => todo!("Integer uniforms are not currently supported"),
			SpirvType::Float(width) => {
				match width {
					32 => UniformBufferElement::Float,

					_ => panic!()
				}
			},
			SpirvType::Vector(id, count) => {
				match Self::get_struct_format(id, types) {
					UniformBufferElement::Float => {
						match count {
							2 => UniformBufferElement::Vec2,
							3 => UniformBufferElement::Vec3,
							4 => UniformBufferElement::Vec4,

							_ => panic!()
						}
					},

					_ => panic!()
				}
			},
			SpirvType::Matrix(id, count) => {
				match Self::get_struct_format(id, types) {
					UniformBufferElement::Vec2 => {
						match count {
							2 => UniformBufferElement::Mat2,

							_ => panic!()
						}
					},
					UniformBufferElement::Vec3 => {
						match count {
							3 => UniformBufferElement::Mat3,

							_ => panic!()
						}
					},
					UniformBufferElement::Vec4 => {
						match count {
							4 => UniformBufferElement::Mat4,

							_ => panic!()
						}
					},

					_ => panic!()
				}
			},
			SpirvType::Array(_, _) => todo!("Arrays are not currently supported"),
			SpirvType::RuntimeArray(_) => todo!("Runtime sized arrays are not currently supported"),
			SpirvType::Struct(_) => todo!("Structs in uniforms are not currently supported"),

			_ => panic!()
		}
	}
}

impl DecorationSet {
	fn apply_decoration(&mut self, instruction: &[u32]) {
		if instruction[2] == Decorations::Binding as u32 {
			self.binding = Some(instruction[3]);
		} else if instruction[2] == Decorations::DescriptorSet as u32 {
			self.set = Some(instruction[3]);
		} else {
			return;
		}
	}
}

impl UniformBufferElement {
	pub(crate) fn size(uniforms: &[UniformBufferElement]) -> u32 {
		let mut sum = 0;

		for uniform in uniforms {
			sum += match uniform {
				UniformBufferElement::Float => 4,
				UniformBufferElement::Vec2 => 4 * 2,
				UniformBufferElement::Vec3 => 4 * 3,
				UniformBufferElement::Vec4 => 4 * 4,
				UniformBufferElement::Mat2 => 4 * 2 * 2,
				UniformBufferElement::Mat3 => 4 * 3 * 3,
				UniformBufferElement::Mat4 => 4 * 4 * 4,
			}
		}
		
		return sum;
	}
}

#[repr(u16)]
enum OpCode {
	// Misc. Instructions
	OpNop					= 0,

	// Annotation Instructions
	OpDecorate 				= 71,

	// Type-Declaration Instructions
	OpTypeVoid				= 19,
	OpTypeBool				= 20,
	OpTypeInt				= 21,
	OpTypeFloat				= 22,
	OpTypeVector			= 23,
	OpTypeMatrix			= 24,
	OpTypeImage				= 25,
	OpTypeSampler			= 26,
	OpTypeSampledImage 		= 27,
	OpTypeArray				= 28,
	OpTypeRuntimeArray		= 29,
	OpTypeStruct			= 30,
	OpTypePointer			= 32,
	OpTypeFunction			= 33,

	// Constant-Declaration Instructions
	OpConstantTrue			= 41,
	OpConstantFalse			= 42,
	OpConstant				= 43,

	// Memory Instructions
	OpVariable 				= 59,
}

impl OpCode {
	fn get_op_code(code: u16) -> Self {
		match code {
			0  => OpCode::OpNop,

			71 => OpCode::OpDecorate,

			19 => OpCode::OpTypeVoid,
			20 => OpCode::OpTypeBool,
			21 => OpCode::OpTypeInt,
			22 => OpCode::OpTypeFloat,
			23 => OpCode::OpTypeVector,
			24 => OpCode::OpTypeMatrix,
			25 => OpCode::OpTypeImage,
			26 => OpCode::OpTypeSampler,
			27 => OpCode::OpTypeSampledImage,
			28 => OpCode::OpTypeArray,
			29 => OpCode::OpTypeRuntimeArray,
			30 => OpCode::OpTypeStruct,
			32 => OpCode::OpTypePointer,
			33 => OpCode::OpTypeFunction,

			41 => OpCode::OpConstantTrue,
			42 => OpCode::OpConstantFalse,
			43 => OpCode::OpConstant,

			59 => OpCode::OpVariable,

			_  => OpCode::OpNop,
		}
	}
}

#[repr(u32)]
enum Decorations {
	Binding 		= 33,
	DescriptorSet 	= 34,
}

#[derive(Debug)]
#[derive(PartialEq, Eq)]
#[repr(u32)]
enum StorageClass {
	UniformConstant 	= 0,
	Input				= 1,
	Uniform 			= 2,
	Output				= 3,
}

impl StorageClass {
	fn from_code(code: u32) -> Self {
		match code {
			0 => Self::UniformConstant,
			1 => Self::Input,
			2 => Self::Uniform,
			3 => Self::Output,
			_ => todo!(),
		}
	}
}