
use std::sync::Arc;

use crate::{
	pipeline::Pipeline, 
	render_engine::{RenderEngine, RenderEngineCreateInfo}, 
	shader::{Shader, ShaderType},
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
/// Ensure `Pipeline::new()` and `Pipeline::drop()` are working as expected.
fn new_pipeline() {
	let create_info = RenderEngineCreateInfo::new()
		.with_spirv_compiler();
	let engine = RenderEngine::new(create_info).unwrap();

	let vertex_binary = Shader::compile(&engine, "vertex.glsl", ShaderType::Vertex, VERTEX_SOURCE).unwrap();
	let vertex_shader = Shader::new(&engine, vertex_binary).unwrap().unwrap();

	let fragment_binary = Shader::compile(&engine, "fragment.glsl", ShaderType::Fragment, FRAGMENT_SOURCE).unwrap();
	let fragment_shader = Shader::new(&engine, fragment_binary).unwrap().unwrap();

	let pipeline = Pipeline::new(&engine, &vec![vertex_shader, fragment_shader]).unwrap().unwrap();

	drop(pipeline);

	let pipeline_list = Pipeline::get_all(&engine).unwrap().unwrap();
	assert!(pipeline_list.len() == 0)
}

#[test]
/// Ensure that `Pipeline::new()` returns Err(()) as expected when shaders of duplicate stages are supplied.
/// VUID-VkGraphicsPipelineShaderGroupsCreateInfoNV-pGroups-02882
fn duplicate_shader_error() {
	let create_info = RenderEngineCreateInfo::new()
		.with_spirv_compiler();
	let engine = RenderEngine::new(create_info).unwrap();

	let vertex_binary = Shader::compile(&engine, "vertex.glsl", ShaderType::Vertex, VERTEX_SOURCE).unwrap();
	let vertex_shader = Shader::new(&engine, vertex_binary).unwrap().unwrap();

	let fragment_binary = Shader::compile(&engine, "fragment.glsl", ShaderType::Fragment, FRAGMENT_SOURCE).unwrap();
	let fragment_shader = Shader::new(&engine, fragment_binary).unwrap().unwrap();

	let result = Pipeline::new(&engine, &vec![vertex_shader.clone(), vertex_shader, fragment_shader]).unwrap();

	assert!(result.unwrap_err() == ());
}

#[test]
/// Ensure that `Pipeline::new()` returns Err(()) if an invalid set of shaders are supplied (i.e. no vertex / mesh shader).
/// VUID-VkGraphicsPipelineShaderGroupsCreateInfoNV-pGroups-02882
fn invalid_shader_set() {
	let create_info = RenderEngineCreateInfo::new()
		.with_spirv_compiler();
	let engine = RenderEngine::new(create_info).unwrap();

	let fragment_binary = Shader::compile(&engine, "fragment.glsl", ShaderType::Fragment, FRAGMENT_SOURCE).unwrap();
	let fragment_shader = Shader::new(&engine, fragment_binary).unwrap().unwrap();

	let result = Pipeline::new(&engine, &vec![fragment_shader]).unwrap();

	assert!(result.unwrap_err() == ());
}

#[test]
/// Ensure `Pipeline::get_all()` is working as expected.
fn get_all() {
	let create_info = RenderEngineCreateInfo::new()
		.with_spirv_compiler();
	let engine = RenderEngine::new(create_info).unwrap();

	let vertex_binary = Shader::compile(&engine, "vertex.glsl", ShaderType::Vertex, VERTEX_SOURCE).unwrap();
	let vertex_shader = Shader::new(&engine, vertex_binary).unwrap().unwrap();

	let fragment_binary = Shader::compile(&engine, "fragment.glsl", ShaderType::Fragment, FRAGMENT_SOURCE).unwrap();
	let fragment_shader = Shader::new(&engine, fragment_binary).unwrap().unwrap();

	let pipeline = Pipeline::new(&engine, &vec![vertex_shader, fragment_shader]).unwrap().unwrap();

	let pipeline_list = Pipeline::get_all(&engine).unwrap().unwrap();

	assert!(pipeline_list.len() == 1);
	assert!(Arc::ptr_eq(&pipeline, &pipeline_list[0]));
}
