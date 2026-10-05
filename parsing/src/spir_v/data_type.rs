
use crate::spir_v::enumerations::{Dim, ImageFormat};

/* TODO
	- [ ] Add Runtime Arrays
 */
#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq, Eq)]
pub enum DataType {
	Void,

	Array { element_type: Box<DataType>, count: usize },
	Struct { members: Box<[DataType]> },

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

	Image { dimension: Dim, pixel_format: Box<DataType>, texture_format: ImageFormat },
	Sampler,
	ImageSampler { dimension: Dim, pixel_format: Box<DataType>, texture_format: ImageFormat },
}

impl DataType {
	pub fn get_size(&self) -> usize {
		match self {
			DataType::Void => 	0,
			DataType::Array { element_type, count } => element_type.get_size() * *count,
			DataType::Struct { members } => members.iter().map(|dt| dt.get_size()).sum(),
			DataType::Float => 	4,
			DataType::Vec2 => 	4*2,
			DataType::Vec3 => 	4*3,
			DataType::Vec4 => 	4*4,
			DataType::Mat2 => 	4*2*2,
			DataType::Mat3 => 	4*3*3,
			DataType::Mat4 => 	4*4*4,
			DataType::Double => 8,
			DataType::DVec2 => 	8*2,
			DataType::DVec3 => 	8*3,
			DataType::DVec4 => 	8*4,
			DataType::DMat2 => 	8*2*2,
			DataType::DMat3 => 	8*3*3,
			DataType::DMat4 => 	8*4*4,
			DataType::Int => 	4,
			DataType::IVec2 => 	4*2,
			DataType::IVec3 => 	4*3,
			DataType::IVec4 => 	4*4,
			DataType::IMat2 => 	4*2*2,
			DataType::IMat3 => 	4*3*3,
			DataType::IMat4 => 	4*4*4,
			DataType::UInt => 	4,
			DataType::UVec2 => 	4*2,
			DataType::UVec3 => 	4*3,
			DataType::UVec4 => 	4*4,
			DataType::UMat2 => 	4*2*2,
			DataType::UMat3 => 	4*3*3,
			DataType::UMat4 => 	4*4*4,
			DataType::Image { dimension: _, pixel_format: _, texture_format: _ } => todo!(),
			DataType::Sampler => todo!(),
			DataType::ImageSampler { dimension: _, pixel_format: _, texture_format: _ } => todo!(),
		}
	}
}
