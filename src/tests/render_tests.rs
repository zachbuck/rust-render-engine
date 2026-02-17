use image::open;

use crate::{
	RenderEngine, 
	RenderEngineCreateInfo, 
	mesh_data::Vertex, 
	shader::ShaderType
};

#[test]
fn basic_triangle() {
	let mut renderer = RenderEngine::new(RenderEngineCreateInfo::default()).unwrap();
	
	let image = renderer.create_image_surface(100, 100).unwrap();

	let vertices = vec![
		Vertex {
			position: [-0.5, -0.25, 0.5],
			normal: [0.0, 0.0, 0.0],
			uv: [0.0, 0.0]
		},
		Vertex {
			position: [0.5, -0.25, 0.5],
			normal: [0.0, 0.0, 0.0],
			uv: [0.0, 0.0]
		},
		Vertex {
			position: [0.0, 0.5, 0.5],
			normal: [0.0, 0.0, 0.0],
			uv: [0.0, 0.0],
		}
	];

	let indices = vec![0, 1, 2];

	let mesh_data = renderer.create_mesh_data(vertices, indices);
	
	let vertex_source = "
		#version 460

		layout(location = 0) in vec3 position;
		layout(location = 1) in vec3 normal;
		layout(location = 2) in vec2 uv;

		void main() {
			gl_Position = vec4(position, 1.0);
		}
	";

	let vertex_shader = renderer.create_shader(vertex_source.to_string(), "vertex.glsl".to_string(), ShaderType::Vertex);

	let fragment_source = "
		#version 460

		layout(location = 0) out vec4 f_color;
		
		void main() {
			f_color = vec4(1.0, 0.0, 0.0, 1.0);
		}
	";

	let fragment_shader = renderer.create_shader(fragment_source.to_string(), "fragment.glsl".to_string(), ShaderType::Fragment);

	let graphics_program = renderer.create_graphics_program(vec![vertex_shader, fragment_shader]);

	let _object = renderer.create_render_object(mesh_data, graphics_program);

	renderer.render_all_render_targets();
	renderer.push_render_calls();

	let data = renderer.get_image_surface_data(image).unwrap();

	let test = open("tests/render_tests/basic_triangle.png").unwrap().into_rgba8();

	assert!(data.dimensions() == test.dimensions(), "data and test case dimensions do not match");

	for x in 0..data.width() {
		for y in 0..data.height() {
			assert!(data.get_pixel(x, y) == test.get_pixel(x, y));
		}
	}
}