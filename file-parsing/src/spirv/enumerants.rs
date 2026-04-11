
#[repr(u16)]
pub(super) enum Instruction {
	// Misc Instructions
	OpNop	= 0,

	// Debug Instructions
	OpName 			= 5,
	OpMemberName	= 6,

	// Annotation Instructions
	OpDecorate			= 71,
	OpMemberDecorate	= 72,
	
	// Mode setting Instructions
	OpEntryPoint	= 15,

	// Type Declaration Instructions
	OpTypeVoid			= 19,
	OpTypeBool			= 20,
	OpTypeInt			= 21,
	OpTypeFloat			= 22,
	OpTypeVector		= 23,
	OpTypeMatrix		= 24,
	OpTypeArray			= 28,
	OpTypeRuntimeArray	= 29,
	OpTypeStruct		= 30,
	OpTypePointer		= 32,

	// Constant Declaration Instructions
	OpConstantTrue		= 41,
	OpConstantFalse		= 42,
	OpConstant			= 43,

	// Memory Instructions
	OpVariable			= 59,
}

impl Instruction {
	pub(super) fn get_instruction(code: u16) -> Self {
		match code {
			 0 => Self::OpNop,

			 5 => Self::OpName,
			 6 => Self::OpMemberName,

			71 => Self::OpDecorate,
			72 => Self::OpMemberDecorate,

			15 => Self::OpEntryPoint,

			19 => Self::OpTypeVoid,
			20 => Self::OpTypeBool,
			21 => Self::OpTypeInt,
			22 => Self::OpTypeFloat,
			23 => Self::OpTypeVector,
			24 => Self::OpTypeMatrix,
			28 => Self::OpTypeArray,
			29 => Self::OpTypeRuntimeArray,
			30 => Self::OpTypeStruct,
			32 => Self::OpTypePointer,
			
			41 => Self::OpConstantTrue,
			42 => Self::OpConstantFalse,
			43 => Self::OpConstant,

			59 => Self::OpVariable,

			_ => Self::OpNop,
		}
	}
}

#[derive(PartialEq, Eq)]
#[derive(Debug)]
#[repr(u32)]
pub(super) enum StorageClass {
	UniformConstant	= 0,
	Input			= 1,
	Uniform			= 2,
	Output			= 3,
	Unknown, 
}

impl StorageClass {
	pub(super) fn get_storage_class(code: u32) -> Self {
		match code {
			0 => Self::UniformConstant,
			1 => Self::Input,
			2 => Self::Uniform,
			3 => Self::Output,
			_ => Self::Unknown,
		}
	}
}

#[repr(u32)]
pub(super) enum Decoration {
	Binding 		= 33,
	DescriptorSet 	= 34,
	Unknown
}

impl Decoration {
	pub(super) fn get_decoration(code: u32) -> Decoration {
		match code {
			33 => Self::Binding,
			34 => Self::DescriptorSet,
			_ => Self::Unknown,
		}
	}
}
