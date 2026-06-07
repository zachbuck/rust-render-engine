
pub enum VertexCollection {
	Vertex2D(Box<[Vertex2D]>),
	Vertex3D(Box<[Vertex3D]>),
}

#[repr(C)]
pub struct Vertex2D {
	pub position: [f32; 2],
	pub uv: [f32; 2],
}

#[repr(C)]
pub struct Vertex3D {
	pub position: [f32; 3],
	pub normal: [f32; 3],
	pub uv: [f32; 2],
}