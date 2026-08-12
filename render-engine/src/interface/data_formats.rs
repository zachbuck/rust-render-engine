
use vulkano::{
	buffer::BufferContents,
	pipeline::graphics::vertex_input::Vertex,
};

#[repr(C)]
#[derive(BufferContents, Vertex)]
#[derive(Debug)]
pub struct Vertex3D {
	#[format(R32G32B32_SFLOAT)]
	pub position: 	[f32; 3],

	#[format(R32G32B32_SFLOAT)]
	pub normal:		[f32; 3],

	#[format(R32G32B32_SFLOAT)]
	pub uv:			[f32; 2],
}