
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
/// Ensure that `Shader::compile()` is working as expected.
fn compile_shader() {
	let create_info = RenderEngineCreateInfo::new()
		.with_spirv_compiler();

	let engine = RenderEngine::new(create_info).unwrap();

	let binary = Shader::compile(&engine, "vertex.glsl", ShaderType::Vertex, VERTEX_SOURCE).unwrap();

	let test_binary = vec![119734787,65536,851979,31,0,131089,1,393227,1,1280527431,1685353262,808793134,0,196622,0,1,589839,0,4,1852399981,0,13,18,27,30,196611,2,460,655364,1197427783,1279741775,1885560645,1953718128,1600482425,1701734764,1919509599,1769235301,25974,524292,1197427783,1279741775,1852399429,1685417059,1768185701,1952671090,6649449,262149,4,1852399981,0,393221,11,1348430951,1700164197,2019914866,0,393222,11,0,1348430951,1953067887,7237481,458758,11,1,1348430951,1953393007,1702521171,0,458758,11,2,1130327143,1148217708,1635021673,6644590,458758,11,3,1130327143,1147956341,1635021673,6644590,196613,13,0,327685,18,1769172848,1852795252,0,262149,27,1836216174,27745,196613,30,30325,196679,11,2,327752,11,0,11,0,327752,11,1,11,1,327752,11,2,11,3,327752,11,3,11,4,262215,18,30,0,262215,27,30,1,262215,30,30,2,131091,2,196641,3,2,196630,6,32,262167,7,6,4,262165,8,32,0,262187,8,9,1,262172,10,6,9,393246,11,7,6,10,10,262176,12,3,11,262203,12,13,3,262165,14,32,1,262187,14,15,0,262167,16,6,3,262176,17,1,16,262203,17,18,1,262187,6,20,1065353216,262176,25,3,7,262203,17,27,1,262167,28,6,2,262176,29,1,28,262203,29,30,1,327734,2,4,0,3,131320,5,262205,16,19,18,327761,6,21,19,0,327761,6,22,19,1,327761,6,23,19,2,458832,7,24,21,22,23,20,327745,25,26,13,15,196670,26,24,65789,65592];

	assert!(binary.len() == test_binary.len());
	for x in 0..binary.len() {
		assert!(binary[x] == test_binary[x])
	}
}

#[test]
/// Ensure that `Shader::new()` and `Shader::drop()` are working as expected.
fn new_shader() {
	let create_info = RenderEngineCreateInfo::new()
		.with_spirv_compiler();

	let engine = RenderEngine::new(create_info).unwrap();

	let binary = Shader::compile(&engine, "vertex.glsl", ShaderType::Vertex, VERTEX_SOURCE).unwrap();
	let shader = Shader::new(&engine, binary).unwrap().unwrap();

	drop(shader);

	
}