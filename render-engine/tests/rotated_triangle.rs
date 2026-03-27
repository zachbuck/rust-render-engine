
use std::{
	fs::File, 
	io::Read,
};

use render_engine::{
	mesh_data::{MeshData, Vertex3D}, 
	pipeline::Pipeline, 
	render_engine::{RenderEngine, RenderEngineCreateInfo}, 
	render_surface::image_surface::ImageSurface, 
	renderable::render_object::RenderObject, 
	shader::{Shader, ShaderType},
};

const VERTICES: [Vertex3D; 3] = [
	Vertex3D { position: [ 0.5, 0.375, 0.5], normal: [0.0; 3], uv: [0.0; 2] },
	Vertex3D { position: [ 0.0,-0.375, 0.5], normal: [0.0; 3], uv: [0.0; 2] },
	Vertex3D { position: [-0.5, 0.375, 0.5], normal: [0.0; 3], uv: [0.0; 2] },
];

const INDICES: [u16; 3] = [
	0, 1, 2
];

const VERTEX_PATH: &str = "./tests/rotated_triangle/vertex.glsl.vert";
const FRAGMENT_PATH: &str = "./tests/rotated_triangle/fragment.glsl.frag";

#[test]
#[allow(unused)]
fn render_scene() {
	let create_info = RenderEngineCreateInfo::new()
		.with_app_name("Rotated Triangle".to_string())
		.with_app_vers(0, 1, 0)
		.with_spirv_compiler();
	let engine = RenderEngine::new(create_info).unwrap();

	let image_surface = ImageSurface::new(&engine, 100, 100).unwrap().unwrap();

	let mesh_data = MeshData::new(&engine, VERTICES.to_vec(), INDICES.to_vec()).unwrap().unwrap();

	let mut file = File::open(VERTEX_PATH).unwrap();
	let mut source = String::new();
	file.read_to_string(&mut source).unwrap();
	let binary = Shader::compile(&engine, VERTEX_PATH, ShaderType::Vertex, &source).unwrap();
	let vertex_shader = Shader::new(&engine, binary).unwrap().unwrap();

	let mut file = File::open(FRAGMENT_PATH).unwrap();
	let mut source = String::new();
	file.read_to_string(&mut source).unwrap();
	let binary = Shader::compile(&engine, FRAGMENT_PATH, ShaderType::Fragment, &source).unwrap();
	let fragment_shader = Shader::new(&engine, binary).unwrap().unwrap();

	let pipeline = Pipeline::new(&engine, &[vertex_shader, fragment_shader]).unwrap().unwrap();

	let render_object = RenderObject::new(&engine, mesh_data, pipeline).unwrap().unwrap();
}
