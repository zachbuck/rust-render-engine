use std::{fs::File, io::Read};

use image::open;

use crate::{
	RenderEngine, 
	RenderEngineCreateInfo, 
	mesh_data::Vertex, 
	shader::ShaderType
};

#[test]
fn triangle() {
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

	let mesh_data = renderer.create_mesh_data(vertices, indices).unwrap();
	
	let vertex_path = "tests/test_scenes/triangle/triangle.vert";
	let mut file = File::open(vertex_path).unwrap();
	let mut vertex_source = String::new();
	file.read_to_string(&mut vertex_source).unwrap();
	let (vertex_binary, _) = renderer.compile_shader(vertex_source, vertex_path.to_string(), ShaderType::Vertex).unwrap();
	let vertex_shader = renderer.create_shader(vertex_binary).unwrap();

	let fragment_path = "tests/test_scenes/triangle/triangle.frag";
	let mut file = File::open(fragment_path).unwrap();
	let mut fragment_source = String::new();
	file.read_to_string(&mut fragment_source).unwrap();
	let (fragment_binary, _) = renderer.compile_shader(fragment_source, fragment_path.to_string(), ShaderType::Fragment).unwrap();
	let fragment_shader = renderer.create_shader(fragment_binary).unwrap();

	let graphics_program = renderer.create_graphics_program(vec![vertex_shader, fragment_shader]);

	let _object = renderer.create_render_object(mesh_data, graphics_program).unwrap();

	renderer.render_all_render_targets().unwrap();
	renderer.push_render_calls().unwrap();

	let data = renderer.get_image_surface_data(image).unwrap();

	let test = open("tests/test_scenes/triangle/test.png").unwrap().into_rgba8();

	assert!(data.dimensions() == test.dimensions(), "data and test case dimensions do not match");

	for x in 0..data.width() {
		for y in 0..data.height() {
			assert!(data.get_pixel(x, y) == test.get_pixel(x, y));
		}
	}
}