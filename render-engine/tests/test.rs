
use std::sync::Arc;

use render_engine::{
	RenderEngine, RenderEngineCreateInfo, RenderingBackend, 
	data_format::{
		pixel::RGBA8, 
		vertex::Vertex2D,
	}, 
	engine_future::EngineFuture, 
	instruction_buffer::InstructionBufferBuilder, 
	mesh_data::MeshData, 
	pipeline::GraphicsPipeline, 
	render_target::render_object::RenderObject, 
	shader::Shader, 
	surface::image_surface::ImageSurface,
};

const VERTICES: [Vertex2D; 4] = [
	Vertex2D {position: [ 0.5, 0.5], uv: [ 1.0, 1.0] },
	Vertex2D {position: [-0.5, 0.5], uv: [-1.0, 1.0] },
	Vertex2D {position: [-0.5,-0.5], uv: [-1.0,-1.0] },
	Vertex2D {position: [ 0.5,-0.5], uv: [ 1.0,-1.0] },
];

const INDICES: [u32; 6] = [
	0, 1, 2,
	0, 2, 3,
];

#[test]
fn main() {
	let render_engine = RenderEngine::new(RenderEngineCreateInfo::default(RenderingBackend::Vulkan)).unwrap();

	let surface: Arc<ImageSurface<RGBA8>> = ImageSurface::new(&render_engine, [100, 100]).unwrap().unwrap();

	let mesh_data = MeshData::new(&render_engine, Box::new(VERTICES), Box::new(INDICES)).unwrap().unwrap();

	let vertex_shader = Shader::from_spirv(&render_engine, Box::new([])).unwrap().unwrap();
	let fragment_shader = Shader::from_spirv(&render_engine, Box::new([])).unwrap().unwrap();

	let shaders = Box::new([vertex_shader, fragment_shader]);
	let pipeline = GraphicsPipeline::new(&surface, shaders).unwrap().unwrap();

	let render_object = RenderObject::new(&render_engine, mesh_data, pipeline).unwrap().unwrap();

	let mut instruction_buffer = InstructionBufferBuilder::new();

	instruction_buffer
		.bind_surface(&surface).unwrap()
		.draw(&render_object).unwrap();

	let instruction_buffer = instruction_buffer.build();
	instruction_buffer.submit(&render_engine).unwrap().unwrap();
}
