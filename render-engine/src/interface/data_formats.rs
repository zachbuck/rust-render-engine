
use parsing::{
	meshes::obj::RawObjVertex, 
	spir_v::data_type::DataType,
};
use vulkano::{
	buffer::BufferContents,
	pipeline::graphics::vertex_input::Vertex as VulkanoVertex,
};

/* TODO
	- [ ] derive vulkano::pipeline::graphics::vertex_input::Vertex for V: crate::data_formats::Vertex
*/
pub trait Vertex {
	const VERTEX_FORMAT: &'static[DataType];
}

#[repr(C)]
#[derive(BufferContents, VulkanoVertex)]
#[derive(Debug)]
pub struct Vertex3D {
	#[format(R32G32B32_SFLOAT)]
	pub position: 	[f32; 3],

	#[format(R32G32B32_SFLOAT)]
	pub normal:		[f32; 3],

	#[format(R32G32_SFLOAT)]
	pub uv:			[f32; 2],
}

impl From<RawObjVertex> for Vertex3D {
	fn from(value: RawObjVertex) -> Self {
		let normal = value.normal.unwrap_or([0.0, 0.0, 0.0]);
		let texture = value.texture.unwrap_or([0.0, 0.0, 0.0]);

		Vertex3D {
			position: [value.position[0], value.position[1], value.position[2]],
			normal: normal,
			uv: [texture[0], texture[1]],
		}
	}
}

impl Vertex for Vertex3D {
	const VERTEX_FORMAT: &'static[DataType] = &[DataType::Vec3, DataType::Vec3, DataType::Vec2];
}
