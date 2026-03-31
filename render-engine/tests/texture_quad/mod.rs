
use std::{
	fs::File, 
	io::Read
};

use image::{ImageBuffer, Rgba, open};
use render_engine::{
	mesh_data::{MeshData, Vertex3D}, 
	pipeline::Pipeline, 
	render_engine::{
		engine_future::EngineFuture,
		RenderEngine, 
		RenderEngineFlags
	}, 
	render_surface::image_surface::ImageSurface, 
	renderable::{
		descriptor_set_data::DescriptorData, render_object::RenderObject
	}, 
	shader::{Shader, ShaderType}, 
	texture::Texture,
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

const VERTEX_PATH: &str = "./tests/texture_quad/vertex.glsl.vert";
const FRAGMENT_PATH: &str = "./tests/texture_quad/fragment.glsl.frag";

const TEXTURE_PATH: &str = "./tests/texture_quad/texture.png";

const TEST_IMAGE_PATH: &str = "./tests/texture_quad/test.png";

pub fn render_scene() {
	let flags = RenderEngineFlags {
		feature_spirv_compiler: true,
		..Default::default()
	};
	let engine = RenderEngine::new("Texture Quad Test", [0, 1, 0], flags).unwrap();

	let image_surface = ImageSurface::new(&engine, 200, 200).wait().unwrap();
	
	let mesh_data = MeshData::new(&engine, VERTICES.to_vec(), INDICES.to_vec()).wait().unwrap();

	let mut file = File::open(VERTEX_PATH).unwrap();
	let mut source = String::new();
	file.read_to_string(&mut source).unwrap();
	let binary = Shader::compile(&engine, VERTEX_PATH, ShaderType::Vertex, &source).unwrap();
	let vertex_shader = Shader::new(&engine, binary).wait().unwrap();

	let mut file = File::open(FRAGMENT_PATH).unwrap();
	let mut source = String::new();
	file.read_to_string(&mut source).unwrap();
	let binary = Shader::compile(&engine, FRAGMENT_PATH, ShaderType::Fragment, &source).unwrap();
	let fragment_shader = Shader::new(&engine, binary).wait().unwrap();

	let pipeline = Pipeline::new(&engine, &vec![vertex_shader, fragment_shader]).wait().unwrap();

 	let texture_data = open(TEXTURE_PATH).unwrap().into_rgba8();
	let texture = Texture::new(&engine, &texture_data, texture_data.width(), texture_data.height()).wait().unwrap();

	let render_object = RenderObject::new(&engine, mesh_data, pipeline).wait().unwrap();

	render_object.update_descriptor(0, 0, DescriptorData::CombinedImageSampler(texture)).wait().unwrap();

	image_surface.render_all().wait().unwrap();

	let image_data = image_surface.get_image_surface_data().wait().unwrap();
	let image = ImageBuffer::<Rgba<u8>, _>::from_raw(200, 200, image_data).unwrap();

	let test_image = open(TEST_IMAGE_PATH).unwrap().into_rgba8();

	assert!(image.dimensions() == test_image.dimensions());
	for x in 0..image.width() {
		for y in 0..image.height() {
			assert!(image.get_pixel(x, y) == test_image.get_pixel(x, y))
		}
	}
}
