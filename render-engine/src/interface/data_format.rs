
pub mod vertex {
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
}

pub mod pixel {
	#[repr(C)]
	pub struct RGBA8 {
		pub r: u8,
		pub g: u8,
		pub b: u8,
		pub a: u8,
	}
}
