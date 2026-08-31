
use macros::EnumFromBackingType;

#[derive(EnumFromBackingType)]
#[repr(u32)]
// 3.2.2
pub enum ExecutionModel {
	Vertex		= 0x0000_0000,
	Fragment	= 0x0000_0004,
}

#[derive(Debug)]
#[derive(Clone, Copy)]
#[derive(PartialEq, Eq)]
#[derive(EnumFromBackingType)]
#[repr(u32)]
// 3.2.6
pub enum StorageClass {
	#[default]
	Error			= 0xFFFF_FFFF,

	UniformConstant	= 0x0000_0000,
	Input			= 0x0000_0001,
	Uniform			= 0x0000_0002,
	Output			= 0x0000_0003,
}

#[derive(Debug)]
#[derive(Clone, Copy)]
#[derive(EnumFromBackingType)]
#[repr(u32)]
// 3.2.7
pub enum Dim {
	OneD 		= 0x0000_0000,
	TwoD 		= 0x0000_0001,
	ThreeD 		= 0x0000_0002,
	Cube 		= 0x0000_0003,
	Rect 		= 0x0000_0004,
	Buffer 		= 0x0000_0005,
	SubpassData = 0x0000_0006
}

#[derive(Debug)]
#[derive(Clone, Copy)]
#[derive(EnumFromBackingType)]
#[repr(u32)]
// 3.2.10
pub enum ImageFormat {
	Unknown 	= 0x0000_0000,

	RGBA32F		= 0x0000_0001,
	RGBA16F 	= 0x0000_0002,
	R32F		= 0x0000_0003,
	RGBA8		= 0x0000_0004,
	RGBA8Snorm 	= 0x0000_0005,

	RGBA32I		= 0x0000_0015,
	RGBA16I		= 0x0000_0016,
	RGBA8I		= 0x0000_0017,
	R32I		= 0x0000_0018,

	RGBA32UI	= 0x0000_001E,
	RGBA16UI	= 0x0000_001F,
	RGBA8UI		= 0x0000_0020,
	R32UI		= 0x0000_0021,
}

#[derive(EnumFromBackingType)]
#[repr(u32)]
// 3.2.19
pub enum Decoration {
	#[default]
	Error			= 0xFFFF_FFFF,

	Binding 		= 0x0000_0021,
	DescriptorSet 	= 0x0000_0022,
}

#[derive(Debug)]
#[derive(PartialEq, Eq)]
#[derive(Clone, Copy)]
#[derive(EnumFromBackingType)]
#[repr(u16)]
pub enum Instruction {
	// 3.3.1 Misc. Instructions
	#[default]
	OpNop				= 0x0000,

	// 3.3.3 Annotation Instructions
	OpDecorate			= 0x0047,

	// 3.3.5 Mode-Setting Instructions
	OpEntryPoint		= 0x000F,

	// 3.3.6 Type-Declaration Instructions
	OpTypeVoid			= 0x0013,
	OpTypeInt			= 0x0015,
	OpTypeFloat			= 0x0016,
	OpTypeVector		= 0x0017,
	OpTypeMatrix		= 0x0018,
	OpTypeImage			= 0x0019,
	OpTypeSampler		= 0x001A,
	OpTypeSampledImage 	= 0x001B,
	OpTypeArray			= 0x001C,
	OpTypeStruct		= 0x001E,
	OpTypePointer		= 0x0020,

	// 3.3.7 Constant-Creation Instructions
	OpConstant			= 0x002B,

	// 3.3.8 Memory Instructions
	OpVariable			= 0x003B,
}
