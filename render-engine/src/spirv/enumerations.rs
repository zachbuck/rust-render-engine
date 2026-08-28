
use macros::EnumFromBackingType;

#[derive(EnumFromBackingType)]
#[repr(u32)]
// 3.2.2
pub enum ExecutionModel {
	Vertex		= 0x0000,
	Fragment	= 0x0004,
}

#[derive(Debug)]
#[derive(Clone, Copy)]
#[derive(PartialEq, Eq)]
#[derive(EnumFromBackingType)]
#[repr(u32)]
// 3.2.6
pub enum StorageClass {
	Input	= 0x0001,
	Uniform	= 0x0002,
	Output	= 0x0003,
}

#[derive(Debug)]
#[derive(PartialEq, Eq)]
#[derive(Clone, Copy)]
#[derive(EnumFromBackingType)]
#[repr(u16)]
pub enum Instruction {
	// 3.3.1 Misc. Instructions
	#[default]
	OpNop			= 0x00,

	// 3.3.5 Mode-Setting Instructions
	OpEntryPoint	= 0x0F,

	// 3.3.6 Type-Declaration Instructions
	OpTypeInt		= 0x15,
	OpTypeFloat		= 0x16,
	OpTypeVector	= 0x17,
	OpTypeMatrix	= 0x18,
	OpTypeArray		= 0x1C,
	OpTypeStruct	= 0x1E,
	OpTypePointer	= 0x20,

	// 3.3.7 Constant-Creation Instructions
	OpConstant		= 0x2B,

	// 3.3.8 Memory Instructions
	OpVariable		= 0x3B,
}
