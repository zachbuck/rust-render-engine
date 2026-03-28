
use std::{
	fs::File, 
	io::Read
};

use image::{ImageBuffer, Rgba, open};
use render_engine::{
	mesh_data::{MeshData, Vertex3D}, 
	pipeline::Pipeline, 
	render_engine::{RenderEngine, RenderEngineCreateInfo}, 
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

#[test]
fn render_scene() {
	let create_info = RenderEngineCreateInfo::new()
		.with_app_name("Texture Quad".to_string())
		.with_app_vers(0, 1, 0)
		.with_spirv_compiler();
	let engine = RenderEngine::new(create_info).unwrap();

	let image_surface = ImageSurface::new(&engine, 200, 200).unwrap().unwrap();
	
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

 	let texture_data = open(TEXTURE_PATH).unwrap().into_rgba8();
	let texture = Texture::new(&engine, &texture_data, texture_data.width(), texture_data.height()).unwrap().unwrap();

	let render_object = RenderObject::new(&engine, mesh_data, pipeline).unwrap().unwrap();

	render_object.update_descriptor(0, 0, DescriptorData::CombinedImageSampler(texture)).unwrap().unwrap();

	image_surface.render_all().unwrap().unwrap();

	let image_data = image_surface.get_image_surface_data().unwrap().unwrap();
	let image = ImageBuffer::<Rgba<u8>, _>::from_raw(200, 200, image_data).unwrap();
	image.save("./test.png").unwrap();
}
