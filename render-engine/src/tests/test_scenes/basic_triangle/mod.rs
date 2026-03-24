
use std::{
	fs::File, 
	io::Read,
};

use image::{ImageBuffer, Rgba, open};

use crate::{
	mesh_data::{MeshData, Vertex3D}, 
	pipeline::Pipeline, 
	render_engine::{RenderEngine, RenderEngineCreateInfo}, 
	render_surface::image_surface::ImageSurface, 
	renderable::render_object::RenderObject, 
	shader::{Shader, ShaderType}
};

const VERTICES: [Vertex3D; 3] = [
	Vertex3D { position: [ 0.5, 0.375, 0.5], normal: [0.0; 3], uv: [0.0; 2] },
	Vertex3D { position: [ 0.0,-0.375, 0.5], normal: [0.0; 3], uv: [0.0; 2] },
	Vertex3D { position: [-0.5, 0.375, 0.5], normal: [0.0; 3], uv: [0.0; 2] },
];

const INDICES: [u16; 3] = [
	0, 1, 2
];

const VERTEX_PATH: &str = "./src/tests/test_scenes/basic_triangle/vertex.glsl.vert";
const FRAGMENT_PATH: &str = "./src/tests/test_scenes/basic_triangle/fragment.glsl.frag";

const TEST_IMAGE_PATH: &str = "./src/tests/test_scenes/basic_triangle/test.png";

#[test]
fn render_scene() {
	let create_info = RenderEngineCreateInfo::new()
		.with_app_name("Basic Triangle".to_string())
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

	let _render_object = RenderObject::new(&engine, mesh_data, pipeline).unwrap().unwrap();

	image_surface.render_all().unwrap().unwrap();

	let image_data = image_surface.get_image_surface_data().unwrap().unwrap();
	let image = ImageBuffer::<Rgba<u8>, _>::from_raw(100, 100, image_data).unwrap();
	
	let test_image = open(TEST_IMAGE_PATH).unwrap().into_rgba8();

	assert!(image.dimensions() == test_image.dimensions());

	for x in 0..image.width() {
		for y in 0..image.height() {
			assert!(image.get_pixel(x, y) == test_image.get_pixel(x, y));
		}
	}
}
