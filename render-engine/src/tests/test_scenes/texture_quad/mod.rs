
use std::{
	fs::File, 
	io::Read
};

use crate::{
	mesh_data::{MeshData, Vertex3D}, pipeline::Pipeline, render_engine::{RenderEngine, RenderEngineCreateInfo}, render_surface::image_surface::ImageSurface, shader::{Shader, ShaderType}
};

const VERTICES: [Vertex3D; 4] = [
	Vertex3D { position: [-0.5,-0.5, 0.5], normal: [0.0; 3], uv: [0.0, 1.0] },
	Vertex3D { position: [ 0.5,-0.5, 0.5], normal: [0.0; 3], uv: [1.0, 1.0] },
	Vertex3D { position: [-0.5, 0.5, 0.5], normal: [0.0; 3], uv: [0.0, 0.0] },
	Vertex3D { position: [ 0.5, 0.5, 0.5], normal: [0.0; 3], uv: [1.0, 0.0] },
];

const INDICES: [u16; 6] = [
	2, 1, 3,
	2, 1, 0,
];

const VERTEX_PATH: &str = "./src/tests/test_scenes/texture_quad/vertex.glsl.vert";
const FRAGMENT_PATH: &str = "./src/tests/test_scenes/texture_quad/fragment.glsl.frag";

#[test]
fn render_scene() {
	let create_info = RenderEngineCreateInfo::new()
		.with_app_name("Texture Quad".to_string())
		.with_app_vers(1, 0, 0)
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

	let pipeline = Pipeline::new(&engine, &vec![vertex_shader, fragment_shader]).unwrap().unwrap();
}
