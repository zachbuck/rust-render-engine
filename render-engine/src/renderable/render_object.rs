
use std::sync::{
	Arc, 
	mpsc::{Sender, sync_channel}
};

use uuid::Uuid;

use crate::{
	mesh_data::MeshData, 
	pipeline::Pipeline, 
	render_engine::{
		RenderEngine, 
		engine_future::EngineFuture, render_command::RenderEngineCommand,
	}, 
	renderable::{
		descriptor_set_data::DescriptorData, 
		render_object::render_object_command::RenderObjectCommand,
	},
};

pub(crate) mod render_object_command;
pub(crate) mod render_object_internal;

pub struct RenderObject {
	pub(crate) uuid: Uuid,
	command_channel: Arc<Sender<RenderEngineCommand>>,

	pub mesh: Arc<MeshData>,
	pub pipeline: Arc<Pipeline>,
}

impl RenderObject {
	pub fn new(render_engine: &RenderEngine, mesh_data: Arc<MeshData>, pipeline: Arc<Pipeline>) -> EngineFuture<Result<Arc<Self>, ()>> {
		let (send, recv) = sync_channel(1);

		render_engine.command_channel.send(
			RenderObjectCommand::CreateRenderObject { 
				sender: send, 
				mesh_data: mesh_data, 
				pipeline: pipeline, 
				command_channel: render_engine.command_channel.clone(), 
			}.into()
		).unwrap();

		EngineFuture::new_single(recv)
	}

	pub fn update_descriptor(&self, set: u32, binding: u32, data: DescriptorData) -> EngineFuture<Result<(), ()>> {
		let result = self.pipeline.descriptor_requirements.descriptors.iter().find(|((s, b), _, _)| *s == set && *b == binding);
		if result.is_none() { return EngineFuture::new_immediate(Err(())); }
		let (_, descriptor_type, _) = result.unwrap();

		if !data.compatable_with(descriptor_type) { return EngineFuture::new_immediate(Err(())) }

		let (send, recv) = sync_channel(1);

		self.command_channel.send(
			RenderObjectCommand::UpdateDescriptor {
				sender: send,
				uuid: self.uuid,
				set: set,
				binding: binding,
				data: data
			}.into()
		).unwrap();

		EngineFuture::new_single(recv)
	}
}

impl Drop for RenderObject {
	fn drop(&mut self) {
		self.command_channel.send(
			RenderObjectCommand::DropRenderObject { 
				uuid: self.uuid, 
			}.into()
		).unwrap();
	}
}

#[cfg(test)]
mod tests {
    use crate::{
		mesh_data::{MeshData, Vertex3D}, 
		pipeline::Pipeline, 
		render_engine::{RenderEngine, RenderEngineFlags}, 
		renderable::{descriptor_set_data::DescriptorData, render_object::RenderObject}, 
		shader::{Shader, ShaderType}, texture::Texture,
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

	const VERTEX_DESCRIPTOR_SOURCE: &str = "
		#version 460

		layout(location = 0) in vec3 position;
		layout(location = 1) in vec3 normal;
		layout(location = 2) in vec2 uv;

		layout(set = 0, binding = 0) uniform UBO {
			mat4 transform;
		};

		void main() {
			gl_Position = transform * vec4(position, 1.0);
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

		layout(location = 0) out vec4 f_color;

		layout(set = 0, binding = 0) uniform sampler2D color_tex;

		void main() {
			f_color = vec4(texture(color_tex, vec2(0.0, 0.0)));
		}
	";

	const TEXTURE_DATA: [u8; 40000] = [0u8; 100 * 100 * 4];

	#[test]
	/// Ensure that `RenderObject::new()` and `RenderObject::drop()` are working as expected.
	fn new_render_object() {
		let flags = RenderEngineFlags {
			feature_spirv_compiler: true,
			..Default::default()
		};
		let engine = RenderEngine::new("Render Object Test", [0, 1, 0], flags).unwrap();

		let mesh_data = MeshData::new(&engine, VERTICES.to_vec(), INDICES.to_vec()).unwrap().unwrap();

		let vertex_binary = Shader::compile(&engine, "vertex.glsl.vert", ShaderType::Vertex, VERTEX_SOURCE).unwrap();
		let vertex_shader = Shader::new(&engine, vertex_binary).unwrap().unwrap();

		let fragment_binary = Shader::compile(&engine, "fragment.glsl.frag", ShaderType::Fragment, FRAGMENT_SOURCE).unwrap();
		let fragment_shader = Shader::new(&engine, fragment_binary).unwrap().unwrap();

		let pipeline = Pipeline::new(&engine, &[vertex_shader, fragment_shader]).unwrap().unwrap();

		let render_object = RenderObject::new(&engine, mesh_data, pipeline).unwrap().unwrap();

		drop(render_object);
	}

	#[test]
	/// Ensure that `RenderObject::update_descriptor()` is working as expected for uniform buffers.
	fn update_descriptor_uniform_buffer() {
		let flags = RenderEngineFlags {
			feature_spirv_compiler: true,
			..Default::default()
		};
		let engine = RenderEngine::new("Render Object Test", [0, 1, 0], flags).unwrap();

		let mesh_data = MeshData::new(&engine, VERTICES.to_vec(), INDICES.to_vec()).unwrap().unwrap();

		let vertex_binary = Shader::compile(&engine, "vertex.glsl.vert", ShaderType::Vertex, VERTEX_DESCRIPTOR_SOURCE).unwrap();
		let vertex_shader = Shader::new(&engine, vertex_binary).unwrap().unwrap();

		let fragment_binary = Shader::compile(&engine, "fragment.glsl.frag", ShaderType::Fragment, FRAGMENT_SOURCE).unwrap();
		let fragment_shader = Shader::new(&engine, fragment_binary).unwrap().unwrap();

		let pipeline = Pipeline::new(&engine, &[vertex_shader, fragment_shader]).unwrap().unwrap();

		let render_object = RenderObject::new(&engine, mesh_data, pipeline).unwrap().unwrap();

		render_object.update_descriptor(0, 0, DescriptorData::UniformBuffer(Box::new([0u8; 4 * 4 * 4]))).unwrap().unwrap();
	}

	#[test]
	/// Ensure that `RenderObject::update_descriptor()` is working as expected for combined image samplers.
	fn update_descriptor_combined_image_sampler() {
		let flags = RenderEngineFlags {
			feature_spirv_compiler: true,
			..Default::default()
		};
		let engine = RenderEngine::new("Render Object Test", [0, 1, 0], flags).unwrap();

		let mesh_data = MeshData::new(&engine, VERTICES.to_vec(), INDICES.to_vec()).unwrap().unwrap();

		let vertex_binary = Shader::compile(&engine, "vertex.glsl.vert", ShaderType::Vertex, VERTEX_SOURCE).unwrap();
		let vertex_shader = Shader::new(&engine, vertex_binary).unwrap().unwrap();

		let fragment_binary = Shader::compile(&engine, "fragment.glsl.frag", ShaderType::Fragment, FRAGMENT_DESCRIPTOR_SOURCE).unwrap();
		let fragment_shader = Shader::new(&engine, fragment_binary).unwrap().unwrap();

		let pipeline = Pipeline::new(&engine, &[vertex_shader, fragment_shader]).unwrap().unwrap();

		let render_object = RenderObject::new(&engine, mesh_data, pipeline).unwrap().unwrap();

		let texture = Texture::new(&engine, &TEXTURE_DATA, 100, 100).unwrap().unwrap();

		render_object.update_descriptor(0, 0, DescriptorData::CombinedImageSampler(texture)).unwrap().unwrap();
	}

	#[test]
	/// Ensure that `RenderObject::update_descriptor()` is returning `Err(())` as expected on incorrect descriptor submission.
	fn update_descriptor_incorrect_descriptor_type() {
		let flags = RenderEngineFlags {
			feature_spirv_compiler: true,
			..Default::default()
		};
		let engine = RenderEngine::new("Render Object Test", [0, 1, 0], flags).unwrap();

		let mesh_data = MeshData::new(&engine, VERTICES.to_vec(), INDICES.to_vec()).unwrap().unwrap();

		let vertex_binary = Shader::compile(&engine, "vertex.glsl.vert", ShaderType::Vertex, VERTEX_SOURCE).unwrap();
		let vertex_shader = Shader::new(&engine, vertex_binary).unwrap().unwrap();

		let fragment_binary = Shader::compile(&engine, "fragment.glsl.frag", ShaderType::Fragment, FRAGMENT_DESCRIPTOR_SOURCE).unwrap();
		let fragment_shader = Shader::new(&engine, fragment_binary).unwrap().unwrap();

		let pipeline = Pipeline::new(&engine, &[vertex_shader, fragment_shader]).unwrap().unwrap();

		let render_object = RenderObject::new(&engine, mesh_data, pipeline).unwrap().unwrap();

		let result = render_object.update_descriptor(0, 0, DescriptorData::UniformBuffer(Box::new([0u8; 4 * 4 * 4]))).unwrap();

		assert!(result.is_err_and(|e| e == ()));
	}
}
