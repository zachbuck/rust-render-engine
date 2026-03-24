
use crate::{
	mesh_data::{MeshData, Vertex3D}, 
	pipeline::Pipeline, 
	render_engine::{RenderEngine, RenderEngineCreateInfo}, 
	renderable::render_object::RenderObject, 
	shader::{Shader, ShaderType}
};

const VERTICES: [Vertex3D; 4] = [
	Vertex3D { position: [ 0.5, 0.5, 0.5], normal: [0.0; 3], uv: [0.0; 2] }, // bottom right
	Vertex3D { position: [-0.5, 0.5, 0.5], normal: [0.0; 3], uv: [0.0; 2] }, // bottom left
	Vertex3D { position: [-0.5,-0.5, 0.5], normal: [0.0; 3], uv: [0.0; 2] }, // top left
	Vertex3D { position: [ 0.5,-0.5, 0.5], normal: [0.0; 3], uv: [0.0; 2] }  // top right
];

const INDICES: [u16; 6] = [
	0, 2, 1,
	0, 3, 2
];

const VERTEX_SOURCE: &str = "
#version 460

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;

void main() {
	gl_Position = vec4(position, 1.0);
}
";

const FRAGMENT_SOURCE: &str = "
#version 460

layout(location = 0) out vec4 f_color;

void main() {
	f_color = vec4(1.0, 0.0, 0.0, 1.0);
}
";

#[test]
/// Ensure that `RenderObject::new()` and `RenderObject::drop()` are working as expected.
fn new_render_object() {
	let create_info = RenderEngineCreateInfo::new()
		.with_spirv_compiler();
	let engine = RenderEngine::new(create_info).unwrap();

	let mesh_data = MeshData::new(engine.clone(), VERTICES.to_vec(), INDICES.to_vec()).unwrap().unwrap();

	let vertex_binary = Shader::compile(engine.clone(), "vertex.glsl", ShaderType::Vertex, VERTEX_SOURCE).unwrap();
	let vertex_shader = Shader::new(engine.clone(), vertex_binary).unwrap().unwrap();

	let fragment_binary = Shader::compile(engine.clone(), "fragment.glsl", ShaderType::Fragment, FRAGMENT_SOURCE).unwrap();
	let fragment_shader = Shader::new(engine.clone(), fragment_binary).unwrap().unwrap();

	let pipeline = Pipeline::new(engine.clone(), &vec![vertex_shader, fragment_shader]).unwrap().unwrap();

	let _render_object = RenderObject::new(engine.clone(), mesh_data, pipeline).unwrap().unwrap();
}
