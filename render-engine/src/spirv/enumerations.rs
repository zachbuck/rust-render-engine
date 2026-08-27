
use macros::EnumFromBackingType;

#[derive(EnumFromBackingType)]
#[repr(u32)]
// 3.2.2
pub enum ExecutionModel {
	#[default]
	Vertex		= 0x0000,
	Fragment	= 0x0004,
}

#[derive(EnumFromBackingType)]
#[derive(Debug)]
#[repr(u32)]
// 3.2.6
pub enum StorageClass {
	#[default]
	Input	= 0x0001,
	Uniform	= 0x0002,
	Output	= 0x0003,
}

#[derive(EnumFromBackingType)]
#[derive(Debug)]
#[derive(PartialEq, Eq)]
#[derive(Clone, Copy)]
#[repr(u16)]
pub enum Instruction {
	// 3.3.1 Misc. Instructions
	#[default]
	OpNop			= 0x00,

	// 3.3.5 Mode-Setting Instructions
	OpEntryPoint	= 0x0F,

	// 3.3.8 Memory Instructions
	OpVariable		= 0x3B
}
