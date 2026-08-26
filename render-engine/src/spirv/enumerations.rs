
use macros::EnumFromBackingType;

#[repr(u32)]
pub enum ExecutionModel {
	Vertex		= 0x0000,
	Fragment	= 0x0004,
}

#[derive(EnumFromBackingType)]
#[repr(u16)]
pub enum Instruction {
	// Misc. Instructions
	#[default]
	OpNop	= 0x00
}
