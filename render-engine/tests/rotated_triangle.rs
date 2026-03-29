
use std::{
	fs::File, 
	io::Read,
};

use image::{ImageBuffer, Rgba, open};
use render_engine::{
	mesh_data::{MeshData, Vertex3D}, 
	pipeline::Pipeline, 
	render_engine::{RenderEngine, RenderEngineFlags}, 
	render_surface::image_surface::ImageSurface, 
	renderable::{descriptor_set_data::DescriptorData, render_object::RenderObject}, 
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

const UNIT_MATRIX: [[f32; 4]; 4] = [
	[1.0, 0.0, 0.0, 0.0],
	[0.0, 1.0, 0.0, 0.0],
	[0.0, 0.0, 1.0, 0.0],
	[0.0, 0.0, 0.0, 1.0],
];

const FLIP_MATRIX: [[f32; 4]; 4] = [
	[1.0, 0.0, 0.0, 0.0],
	[0.0,-1.0, 0.0, 0.0],
	[0.0, 0.0, 1.0, 0.0],
	[0.0, 0.0, 0.0, 1.0],
];

const UNIT_TEST_IMAGE_PATH: &str = "./tests/rotated_triangle/unit_matrix_test.png";
const FLIP_TEST_IMAGE_PATH: &str = "./tests/rotated_triangle/flip_matrix_test.png";

#[test]
fn render_scene() {
	let flags = RenderEngineFlags {
		feature_spirv_compiler: true,
		..Default::default()
	};
	let engine = RenderEngine::new("Rotated Triangle", [0, 1, 0], flags).unwrap();

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

	let mut data = Box::new([0u8; 4 * 4 * 4]);
	for x in 0..4 {
		for y in 0..4 {
			let bytes = UNIT_MATRIX[x][y].to_le_bytes();
			for b in 0..4 {
				data[x * 4 * 4 + y * 4 + b] = bytes[b];
			}
		}
	}

	render_object.update_descriptor(0, 0, DescriptorData::UniformBuffer(data)).unwrap().unwrap();

	image_surface.render_all().unwrap().unwrap();

	let unit_image_data = image_surface.get_image_surface_data().unwrap().unwrap();
	let image = ImageBuffer::<Rgba<u8>, _>::from_raw(image_surface.x_size, image_surface.y_size, unit_image_data).unwrap();
	
	let test_image = open(UNIT_TEST_IMAGE_PATH).unwrap().into_rgba8();

	assert!(image.dimensions() == test_image.dimensions());
	for x in 0..image.width() {
		for y in 0..image.height() {
			assert!(image.get_pixel(x, y) == test_image.get_pixel(x, y))
		}
	}

	let mut data = Box::new([0u8; 4 * 4 * 4]);
	for x in 0..4 {
		for y in 0..4 {
			let bytes = FLIP_MATRIX[x][y].to_le_bytes();
			for b in 0..4 {
				data[x * 4 * 4 + y * 4 + b] = bytes[b];
			}
		}
	}

	render_object.update_descriptor(0, 0, DescriptorData::UniformBuffer(data)).unwrap().unwrap();

	image_surface.render_all().unwrap().unwrap();

	let flip_image_data = image_surface.get_image_surface_data().unwrap().unwrap();
	let image = ImageBuffer::<Rgba<u8>, _>::from_raw(image_surface.x_size, image_surface.y_size, flip_image_data).unwrap();
	let test_image = open(FLIP_TEST_IMAGE_PATH).unwrap().into_rgba8();

	assert!(image.dimensions() == test_image.dimensions());
	for x in 0..image.width() {
		for y in 0..image.height() {
			assert!(image.get_pixel(x, y) == test_image.get_pixel(x, y));
		}
	}
}
