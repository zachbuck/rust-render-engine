
pub struct ObjParser<T> {
	pub vertices: Box<[Vertex]>,
	pub indices: Box<[T]>,
}

pub struct Vertex {
	pub position: [f32; 3],
	pub normal: [f32; 3],
	pub uv: [f32; 3],
}

impl<T> ObjParser<T> {
	pub fn parse<T> 
}
