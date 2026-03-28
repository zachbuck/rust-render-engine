
use std::{
	collections::HashSet, 
	sync::{
		Arc, 
		mpsc::sync_channel
	}
};

use uuid::Uuid;

use crate::{
	pipeline::pipeline_command::PipelineCommand, 
	render_engine::{
		RenderEngine, 
		engine_future::EngineFuture
	}, 
	shader::{Shader, ShaderType, descriptor_requirements::DescriptorRequirements}
};

pub(crate) mod pipeline_command;
pub(crate) mod pipeline_internal;

#[derive(Debug)]
pub struct Pipeline {
	pub(crate) uuid: Uuid,
	render_engine: Arc<RenderEngine>,

	pub shaders: Box<[Arc<Shader>]>,
	pub(crate) descriptor_requirements: DescriptorRequirements,
}

impl Pipeline {
	pub fn new(render_engine: &Arc<RenderEngine>, shaders: &[Arc<Shader>]) -> EngineFuture<Result<Arc<Pipeline>, ()>> {
		// Do some error checking on the input to make sure that this will link into a valid GraphicsPipeline
		// 1. Only one of each stage
		// 2. Must contain Vertex Shader
		// 3. Descriptors must be compatible
		// 4. ... (more requirements but they come when more shader types implemented)

		let mut stages = HashSet::new();
		for shader in shaders {
			if stages.contains(&shader.shader_type) {
				return EngineFuture::new_immediate(Err(()));
			}

			stages.insert(shader.shader_type);
		}

		if !stages.contains(&ShaderType::Vertex) {
			return EngineFuture::new_immediate(Err(()));
		}

		if !DescriptorRequirements::test_compatibility(&shaders.iter().map(|s| s.descriptor_requirements.clone()).collect::<Vec<_>>()) {
			return EngineFuture::new_immediate(Err(()));
		}

		let (send, recv) = sync_channel(1);

		let shaders = shaders.to_owned().into_boxed_slice();

		render_engine.command_channel.send(
			PipelineCommand::CreatePipeline { 
				sender: send, 
				shaders: shaders,
				engine: render_engine.clone(),
			}.into()
		).unwrap();

		return EngineFuture::new_single(recv);
	}

	pub fn get_all(render_engine: &Arc<RenderEngine>) -> EngineFuture<Result<Box<[Arc<Pipeline>]>, ()>> {
		let (send, recv) = sync_channel(1);

		render_engine.command_channel.send(
			PipelineCommand::GetPipelines {
				sender: send,
			}.into()
		).unwrap();
		
		EngineFuture::new_single(recv)
	}
}

impl Drop for Pipeline {
	fn drop(&mut self) {
		self.render_engine.command_channel.send(
			PipelineCommand::DropPipeline { 
				uuid: self.uuid 
			}.into()
		).unwrap()
	}
}

#[cfg(test)]
mod tests {
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

	const VERTEX_DESCRIPTOR_SOURCE: &str = "
		#version 460

		layout(location = 0) in vec3 position;
		layout(location = 1) in vec3 normal;
		layout(location = 2) in vec2 uv;

		layout(location = 0) out vec2 uv_out;

		layout(set = 0, binding = 0) uniform UBO {
			mat4 transform;
		};

		void main() {
			gl_Position = transform * vec4(position, 1.0);
			uv_out = uv;
		}
	";

	const FRAGMENT_SOURCE: &str = "
		#version 460

		layout(location = 0) out vec4 f_color;

		void main() {
			f_color = vec4(1.0, 0.0, 0.0, 1.0);
		}
	";

	const FRAGMENT_DESCRIPTOR_SOURCE: &str = "
		#version 460

		layout(location = 0) in vec2 uv;

		layout(location = 0) out vec4 f_color;

		layout(set = 0, binding = 0) uniform sampler2D color_tex;

		void main() {
			f_color = vec4(texture(color_tex, uv));
		}
	";

	#[test]
	/// Ensure `Pipeline::new()` and `Pipeline::drop()` are working as expected.
	fn new_pipeline() {
		let create_info = RenderEngineCreateInfo::new()
			.with_spirv_compiler();
		let engine = RenderEngine::new(create_info).unwrap();

		let vertex_binary = Shader::compile(&engine, "vertex.glsl.vert", ShaderType::Vertex, VERTEX_SOURCE).unwrap();
		let vertex_shader = Shader::new(&engine, vertex_binary).unwrap().unwrap();

		let fragment_binary = Shader::compile(&engine, "fragment.glsl.frag", ShaderType::Fragment, FRAGMENT_SOURCE).unwrap();
		let fragment_shader = Shader::new(&engine, fragment_binary).unwrap().unwrap();

		let pipeline = Pipeline::new(&engine, &[vertex_shader, fragment_shader]).unwrap().unwrap();

		drop(pipeline);

		let pipeline_list = Pipeline::get_all(&engine).unwrap().unwrap();
		assert!(pipeline_list.len() == 0);
	}

	#[test]
	/// Ensure that `Pipeline::new()` returns `Err(())` as expected when shaders of duplicate stages are supplied.
	/// 
	/// `VUID-VkGraphicsPipelineCreateInfo-stage-06897`
	fn new_pipeline_duplicate_shader() {
		let create_info = RenderEngineCreateInfo::new()
			.with_spirv_compiler();
		let engine = RenderEngine::new(create_info).unwrap();

		let vertex_binary = Shader::compile(&engine, "vertex.glsl.vert", ShaderType::Vertex, VERTEX_SOURCE).unwrap();
		let vertex_shader = Shader::new(&engine, vertex_binary).unwrap().unwrap();

		let vertex_2_binary = Shader::compile(&engine, "vertex.glsl.vert", ShaderType::Vertex, VERTEX_SOURCE).unwrap();
		let vertex_2_shader = Shader::new(&engine, vertex_2_binary).unwrap().unwrap();

		let fragment_binary = Shader::compile(&engine, "fragment.glsl.frag", ShaderType::Fragment, FRAGMENT_SOURCE).unwrap();
		let fragment_shader = Shader::new(&engine, fragment_binary).unwrap().unwrap();

		let result = Pipeline::new(&engine, &[vertex_shader, vertex_2_shader, fragment_shader]).unwrap();

		assert!(result.is_err_and(|e| e == ()));
	}

	#[test]
	/// Ensure that `Pipeline::new()` returns `Err(())` as expected when a set of shaders not including a vertex shader is supplied.
	/// 
	/// `VUID-VkGraphicsPipelineCreateInfo-stage-02096`
	fn new_pipeline_no_vertex_shader() {
		let create_info = RenderEngineCreateInfo::new()
			.with_spirv_compiler();
		let engine = RenderEngine::new(create_info).unwrap();

		let fragment_binary = Shader::compile(&engine, "fragment.glsl.frag", ShaderType::Fragment, FRAGMENT_SOURCE).unwrap();
		let fragment_shader = Shader::new(&engine, fragment_binary).unwrap().unwrap();

		let result = Pipeline::new(&engine, &[fragment_shader]).unwrap();

		assert!(result.is_err_and(|e| e == ()));
	}

	#[test]
	/// Ensure that `Pipeline::new()` returns `Err(())` as expected when a set of shaders with incompatible descriptor bindings is supplied.
	/// 
	/// `VUID-VkGraphicsPipelineCreateInfo-layout-00756`
	fn new_pipeline_incompatible_descriptors() {
		let create_info = RenderEngineCreateInfo::new()
			.with_spirv_compiler();
		let engine = RenderEngine::new(create_info).unwrap();

		let vertex_binary = Shader::compile(&engine, "vertex.glsl.vert", ShaderType::Vertex, VERTEX_DESCRIPTOR_SOURCE).unwrap();
		let vertex_shader = Shader::new(&engine, vertex_binary).unwrap().unwrap();

		let fragment_binary = Shader::compile(&engine, "fragment.glsl.frag", ShaderType::Fragment, FRAGMENT_DESCRIPTOR_SOURCE).unwrap();
		let fragment_shader = Shader::new(&engine, fragment_binary).unwrap().unwrap();

		let result = Pipeline::new(&engine, &[vertex_shader, fragment_shader]).unwrap();

		assert!(result.is_err_and(|e| e == ()));
	}

	#[test]
	/// Ensure that `Pipeline::get_all()` is working as expected.
	fn get_all() {
		let create_info = RenderEngineCreateInfo::new()
			.with_spirv_compiler();
		let engine = RenderEngine::new(create_info).unwrap();

		let vertex_binary = Shader::compile(&engine, "vertex.glsl.vert", ShaderType::Vertex, VERTEX_SOURCE).unwrap();
		let vertex_shader = Shader::new(&engine, vertex_binary).unwrap().unwrap();

		let fragment_binary = Shader::compile(&engine, "fragment.glsl.frag", ShaderType::Fragment, FRAGMENT_SOURCE).unwrap();
		let fragment_shader = Shader::new(&engine, fragment_binary).unwrap().unwrap();

		let pipeline = Pipeline::new(&engine, &[vertex_shader, fragment_shader]).unwrap().unwrap();

		let pipeline_list = Pipeline::get_all(&engine).unwrap().unwrap();

		assert!(pipeline_list.len() == 1);
		assert!(Arc::ptr_eq(&pipeline, &pipeline_list[0]));
	}
}
