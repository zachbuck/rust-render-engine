use std::{thread::sleep, time::Duration};

use crate::{render_engine::{RenderEngine, RenderEngineCreateInfo}, shader::{Shader, ShaderType}};


const VERTEX_SOURCE: &str = "
#version 460

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;

void main() {
	gl_Position = vec4(position, 1.0);
}
";

#[test]
fn compile_shader() {
	let create_info = RenderEngineCreateInfo::new()
		.with_spirv_compiler();

	let engine = RenderEngine::new(create_info).unwrap();

	let _binary = Shader::compile(engine.clone(), "vertex.glsl", ShaderType::Vertex, VERTEX_SOURCE).unwrap();
}

#[test]
fn new_shader() {
	let create_info = RenderEngineCreateInfo::new()
		.with_spirv_compiler();

	let engine = RenderEngine::new(create_info).unwrap();

	let binary = Shader::compile(engine.clone(), "vertex.glsl", ShaderType::Vertex, VERTEX_SOURCE).unwrap();
	let _shader = Shader::new(engine.clone(), binary).unwrap().unwrap();
}

#[test]
fn drop_shader() {
	let create_info = RenderEngineCreateInfo::new()
		.with_spirv_compiler();

	let engine = RenderEngine::new(create_info).unwrap();

	let binary = Shader::compile(engine.clone(), "vertex.glsl", ShaderType::Vertex, VERTEX_SOURCE).unwrap();
	let shader = Shader::new(engine.clone(), binary).unwrap().unwrap();

	drop(shader);

	sleep(Duration::from_secs(1));
}
