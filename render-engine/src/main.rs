
use std::{
	thread, 
	time::Duration,
};

use render_engine::{
	data_formats::Vertex3D, engine_future::EngineFuture, mesh_data::MeshData, render_engine::{RenderEngine, RenderEngineBackend, RenderEngineCreateInfo}, render_instruction_buffer::RenderInstructionBufferBuilder, spirv::{compiler::{Compiler, ShaderStage}, interpreter::Interpreter}, surface::{
		RenderPassCreateInfo, 
		window_surface::{WindowSurface, WindowSurfaceCreateInfo},
	},
};

const VERTICES: [Vertex3D; 3] = [
	Vertex3D { position: [ 0.5,-0.5, 0.5], normal: [ 0.0, 0.0, 0.0], uv: [ 0.0, 0.0] },
	Vertex3D { position: [ 0.0, 0.5, 0.5], normal: [ 0.0, 0.0, 0.0], uv: [ 0.0, 0.0] },
	Vertex3D { position: [-0.5,-0.5, 0.5], normal: [ 0.0, 0.0, 0.0], uv: [ 0.0, 0.0] },
];

const INDICES: [u32; 3] = [
	0, 1, 2,
];

const VERTEX_SOURCE: &str = r#"
#version 460

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;

void main() {
	gl_Position = vec4(position, 1.0);
}
"#;

fn main() -> () {
	// let render_engine = RenderEngine::new(RenderEngineCreateInfo::with_backend(RenderEngineBackend::Vulkan)).unwrap();
	
	// let window = WindowSurface::new(
	// 	&render_engine,
	// 	WindowSurfaceCreateInfo {
	// 		clear_color: [0.0, 1.0, 0.0, 1.0],
	// 		..Default::default()
	// 	},
	// 	RenderPassCreateInfo::default(),
	// ).wait().unwrap();

	// let vertices = Box::new(VERTICES);
	// let indices = Box::new(INDICES);
	// let _mesh_data = MeshData::new(&render_engine, vertices, indices).wait().unwrap();

	// let builder = RenderInstructionBufferBuilder::begin(&window);
	// let instruction_buffer = builder.build();
	// render_engine.submit_render_instructions(instruction_buffer).wait().unwrap();

	// thread::sleep(Duration::from_secs(5));

	let compiler = Compiler::new().unwrap();
	let vertex_binary = compiler.compile_from_source("vertex.glsl.vert", ShaderStage::Vertex, VERTEX_SOURCE).unwrap();
	
	Interpreter::get_shader_stage(&vertex_binary.binary);
}
