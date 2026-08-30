
#[derive(Clone)]
#[derive(Debug)]
pub enum DataType {
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
}
