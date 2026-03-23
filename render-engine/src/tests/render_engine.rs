
use std::{
	thread::sleep, 
	time::Duration,
};

use crate::{
	render_engine::{RenderEngine, RenderEngineCreateInfo}, 
	shader::{Shader, ShaderType}
};

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
fn new_render_engine() {
	let create_info = RenderEngineCreateInfo::new()
		.with_app_name("Test".to_string())
		.with_app_vers(10, 10, 10)
		.with_spirv_compiler();
	let _engine = RenderEngine::new(create_info).unwrap();
}

#[test]
fn drop_render_engine() {
	let create_info = RenderEngineCreateInfo::new();
	let engine = RenderEngine::new(create_info).unwrap();

	drop(engine);

	sleep(Duration::from_secs(1));
}

#[test]
fn with_spirv_compiler() {
	let create_info = RenderEngineCreateInfo::new()
		.with_spirv_compiler();
	let engine = RenderEngine::new(create_info).unwrap();
	let result = Shader::compile(engine.clone(), "vertex.glsl", ShaderType::Vertex, VERTEX_SOURCE);
	assert!(result.is_ok());
}

#[test]
fn without_spirv_compiler() {
	let create_info = RenderEngineCreateInfo::new();
	let engine = RenderEngine::new(create_info).unwrap();
	let result = Shader::compile(engine.clone(), "vertex.glsl", ShaderType::Vertex, VERTEX_SOURCE);
	assert!(result.is_err());
}