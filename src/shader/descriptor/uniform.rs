
#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq, Eq)]
pub enum UniformType {
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

	UInt,
	UVec2,
	UVec3,
	UVec4,

	Bool,
	BVec2,
	BVec3,
	BVec4,
}

pub enum Uniform {
	Float	(f32),
	Vec2	(glam::Vec2),
	Vec3	(glam::Vec3),
	Vec4	(glam::Vec4),
	Mat2	(glam::Mat2),
	Mat3	(glam::Mat3),
	Mat4	(glam::Mat4),

	Double	(f64),
	DVec2	(glam::DVec2),
	DVec3	(glam::DVec3),
	DVec4	(glam::DVec4),
	DMat2	(glam::DMat2),
	DMat3	(glam::DMat3),
	DMat4	(glam::DMat4),

	Int		(i32),
	IVec2	(glam::IVec2),
	IVec3	(glam::IVec3),
	IVec4	(glam::IVec4),

	UInt	(u32),
	UVec2	(glam::UVec2),
	UVec3	(glam::UVec3),
	UVec4	(glam::UVec4),

	Bool	(bool),
	BVec2	(glam::BVec2),
	BVec3	(glam::BVec3),
	BVec4	(glam::BVec4),
}

impl Uniform {
	pub(crate) fn from_type(uniform_type: &UniformType) -> Self {
		match uniform_type {
			UniformType::Float => 	Uniform::Float(1.0),
			UniformType::Vec2 => 	Uniform::Vec2(glam::Vec2::ONE),
			UniformType::Vec3 => 	Uniform::Vec3(glam::Vec3::ONE),
			UniformType::Vec4 => 	Uniform::Vec4(glam::Vec4::ONE),
			UniformType::Mat2 => 	Uniform::Mat2(glam::Mat2::IDENTITY),
			UniformType::Mat3 => 	Uniform::Mat3(glam::Mat3::IDENTITY),
			UniformType::Mat4 => 	Uniform::Mat4(glam::Mat4::IDENTITY),

			UniformType::Double => 	Uniform::Double(1.0),
			UniformType::DVec2 => 	Uniform::DVec2(glam::DVec2::ONE),
			UniformType::DVec3 => 	Uniform::DVec3(glam::DVec3::ONE),
			UniformType::DVec4 => 	Uniform::DVec4(glam::DVec4::ONE),
			UniformType::DMat2 => 	Uniform::DMat2(glam::DMat2::IDENTITY),
			UniformType::DMat3 => 	Uniform::DMat3(glam::DMat3::IDENTITY),
			UniformType::DMat4 => 	Uniform::DMat4(glam::DMat4::IDENTITY),

			UniformType::Int => 	Uniform::Int(1),
			UniformType::IVec2 => 	Uniform::IVec2(glam::IVec2::ONE),
			UniformType::IVec3 => 	Uniform::IVec3(glam::IVec3::ONE),
			UniformType::IVec4 => 	Uniform::IVec4(glam::IVec4::ONE),

			UniformType::UInt => 	Uniform::UInt(1),
			UniformType::UVec2 => 	Uniform::UVec2(glam::UVec2::ONE),
			UniformType::UVec3 => 	Uniform::UVec3(glam::UVec3::ONE),
			UniformType::UVec4 => 	Uniform::UVec4(glam::UVec4::ONE),

			UniformType::Bool => 	Uniform::Bool(true),
			UniformType::BVec2 => 	Uniform::BVec2(glam::BVec2::TRUE),
			UniformType::BVec3 => 	Uniform::BVec3(glam::BVec3::TRUE),
			UniformType::BVec4 => 	Uniform::BVec4(glam::BVec4::TRUE),
		}
	}

	pub(crate) fn into_binary(uniforms: &[Uniform]) -> Vec<u8> {
		let mut binary = Vec::new();

		for uniform in uniforms {
			match uniform {
				Uniform::Float(value) 	=> binary.append(&mut value.to_le_bytes().to_vec()),
				Uniform::Vec2(value) 	=> value.to_array().iter().for_each(|f| binary.append(&mut f.to_le_bytes().to_vec())),
				Uniform::Vec3(value) 	=> value.to_array().iter().for_each(|f| binary.append(&mut f.to_le_bytes().to_vec())),
				Uniform::Vec4(value) 	=> value.to_array().iter().for_each(|f| binary.append(&mut f.to_le_bytes().to_vec())),
				Uniform::Mat2(value) 	=> value.to_cols_array().iter().for_each(|f| binary.append(&mut f.to_le_bytes().to_vec())),
				Uniform::Mat3(value) 	=> value.to_cols_array().iter().for_each(|f| binary.append(&mut f.to_le_bytes().to_vec())),
				Uniform::Mat4(value) 	=> value.to_cols_array().iter().for_each(|f| binary.append(&mut f.to_le_bytes().to_vec())),

				Uniform::Double(value) 	=> binary.append(&mut value.to_le_bytes().to_vec()),
				Uniform::DVec2(value) 	=> value.to_array().iter().for_each(|f| binary.append(&mut f.to_le_bytes().to_vec())),
				Uniform::DVec3(value) 	=> value.to_array().iter().for_each(|f| binary.append(&mut f.to_le_bytes().to_vec())),
				Uniform::DVec4(value) 	=> value.to_array().iter().for_each(|f| binary.append(&mut f.to_le_bytes().to_vec())),
				Uniform::DMat2(value) 	=> value.to_cols_array().iter().for_each(|f| binary.append(&mut f.to_le_bytes().to_vec())),
				Uniform::DMat3(value) 	=> value.to_cols_array().iter().for_each(|f| binary.append(&mut f.to_le_bytes().to_vec())),
				Uniform::DMat4(value) 	=> value.to_cols_array().iter().for_each(|f| binary.append(&mut f.to_le_bytes().to_vec())),

				Uniform::Int(value) 		=> binary.append(&mut value.to_le_bytes().to_vec()),
				Uniform::IVec2(value) 	=> value.to_array().iter().for_each(|f| binary.append(&mut f.to_le_bytes().to_vec())),
				Uniform::IVec3(value) 	=> value.to_array().iter().for_each(|f| binary.append(&mut f.to_le_bytes().to_vec())),
				Uniform::IVec4(value) 	=> value.to_array().iter().for_each(|f| binary.append(&mut f.to_le_bytes().to_vec())),

				Uniform::UInt(value) 		=> binary.append(&mut value.to_le_bytes().to_vec()),
				Uniform::UVec2(value) 	=> value.to_array().iter().for_each(|f| binary.append(&mut f.to_le_bytes().to_vec())),
				Uniform::UVec3(value) 	=> value.to_array().iter().for_each(|f| binary.append(&mut f.to_le_bytes().to_vec())),
				Uniform::UVec4(value) 	=> value.to_array().iter().for_each(|f| binary.append(&mut f.to_le_bytes().to_vec())),

				Uniform::Bool(_value) 	=> todo!(),
				Uniform::BVec2(_value) 	=> todo!(),
				Uniform::BVec3(_value) 	=> todo!(),
				Uniform::BVec4(_value) 	=> todo!(),
			}
		}

		return binary;
	}
}