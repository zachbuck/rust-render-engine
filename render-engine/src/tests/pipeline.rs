
use std::{sync::Arc, thread::sleep, time::Duration};

use crate::{
	pipeline::Pipeline, render_engine::{RenderEngine, RenderEngineCreateInfo}, shader::{Shader, ShaderType}
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

const FRAGMENT_SOURCE: &str = "
#version 460

layout(location = 0) out vec4 f_color;

void main() {
	f_color = vec4(1.0, 0.0, 0.0, 1.0);
}
";

#[test]
fn new_pipeline() {
	let create_info = RenderEngineCreateInfo::new()
		.with_spirv_compiler();
	let engine = RenderEngine::new(create_info).unwrap();

	let vertex_binary = Shader::compile(engine.clone(), "vertex.glsl", ShaderType::Vertex, VERTEX_SOURCE).unwrap();
	let vertex_shader = Shader::new(engine.clone(), vertex_binary).unwrap().unwrap();

	let fragment_binary = Shader::compile(engine.clone(), "fragment.glsl", ShaderType::Fragment, FRAGMENT_SOURCE).unwrap();
	let fragment_shader = Shader::new(engine.clone(), fragment_binary).unwrap().unwrap();

	let _pipeline = Pipeline::new(engine.clone(), &vec![vertex_shader, fragment_shader]).unwrap().unwrap();
}

#[test]
fn get_all() {
	let create_info = RenderEngineCreateInfo::new()
		.with_spirv_compiler();
	let engine = RenderEngine::new(create_info).unwrap();

	let vertex_binary = Shader::compile(engine.clone(), "vertex.glsl", ShaderType::Vertex, VERTEX_SOURCE).unwrap();
	let vertex_shader = Shader::new(engine.clone(), vertex_binary).unwrap().unwrap();

	let fragment_binary = Shader::compile(engine.clone(), "fragment.glsl", ShaderType::Fragment, FRAGMENT_SOURCE).unwrap();
	let fragment_shader = Shader::new(engine.clone(), fragment_binary).unwrap().unwrap();

	let pipeline = Pipeline::new(engine.clone(), &vec![vertex_shader, fragment_shader]).unwrap().unwrap();

	let pipeline_list = Pipeline::get_all(engine.clone()).unwrap().unwrap();

	assert!(pipeline_list.len() == 1);
	assert!(Arc::ptr_eq(&pipeline, &pipeline_list[0]));
}

#[test]
fn drop_pipeline() {
	let create_info = RenderEngineCreateInfo::new()
		.with_spirv_compiler();
	let engine = RenderEngine::new(create_info).unwrap();

	let vertex_binary = Shader::compile(engine.clone(), "vertex.glsl", ShaderType::Vertex, VERTEX_SOURCE).unwrap();
	let vertex_shader = Shader::new(engine.clone(), vertex_binary).unwrap().unwrap();

	let fragment_binary = Shader::compile(engine.clone(), "fragment.glsl", ShaderType::Fragment, FRAGMENT_SOURCE).unwrap();
	let fragment_shader = Shader::new(engine.clone(), fragment_binary).unwrap().unwrap();

	let pipeline = Pipeline::new(engine.clone(), &vec![vertex_shader, fragment_shader]).unwrap().unwrap();
	drop(pipeline);

	sleep(Duration::from_secs(1));

	let pipeline_list = Pipeline::get_all(engine.clone()).unwrap().unwrap();

	assert!(pipeline_list.len() == 0);
}
