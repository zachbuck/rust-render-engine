
use std::sync::{
	Arc, 
	mpsc::sync_channel
};

use uuid::Uuid;

use crate::{
	mesh_data::MeshData, 
	pipeline::Pipeline, 
	render_engine::{
		RenderEngine, 
		engine_future::EngineFuture,
	}, 
	renderable::{
		descriptor_set_data::DescriptorData, 
		render_object::render_object_command::RenderObjectCommand
	}, 
};

pub(crate) mod render_object_command;
pub(crate) mod render_object_internal;

pub struct RenderObject {
	pub(crate) uuid: Uuid,
	render_engine: Arc<RenderEngine>,

	pub mesh: Arc<MeshData>,
	pub pipeline: Arc<Pipeline>,
}

impl RenderObject {
	pub fn new(render_engine: &Arc<RenderEngine>, mesh_data: Arc<MeshData>, pipeline: Arc<Pipeline>) -> EngineFuture<Result<Arc<Self>, ()>> {
		let (send, recv) = sync_channel(1);

		render_engine.command_channel.send(
			RenderObjectCommand::CreateRenderObject { 
				sender: send, 
				mesh_data: mesh_data, 
				pipeline: pipeline, 
				render_engine: render_engine.clone(), 
			}.into()
		).unwrap();

		EngineFuture::new_single(recv)
	}

	pub fn update_descriptor(&self, set: u32, binding: u32, data: DescriptorData) -> EngineFuture<Result<(), ()>> {
		let set_requirements = self.pipeline.descriptor_requirements.sets.iter().find(|s| s.set == set);
		if set_requirements.is_none() { return EngineFuture::new_immediate(Err(())) }
		let set_requirements = set_requirements.unwrap();

		let binding_requirements = set_requirements.bindings.iter().find(|b| b.binding == binding);
		if binding_requirements.is_none() { return EngineFuture::new_immediate(Err(())) }
		let binding_requirements =binding_requirements.unwrap();

		if !data.comptable_with_type(binding_requirements.descriptor_type) { return EngineFuture::new_immediate(Err(())) }

		let (send, recv) = sync_channel(1);

		self.render_engine.command_channel.send(
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
		self.render_engine.command_channel.send(
			RenderObjectCommand::DropRenderObject { 
				uuid: self.uuid, 
			}.into()
		).unwrap();
	}
}

#[cfg(test)]
mod tests {
    use crate::{
		mesh_data::{MeshData, Vertex3D}, pipeline::Pipeline, render_engine::{RenderEngine, RenderEngineCreateInfo}, renderable::render_object::RenderObject, shader::{Shader, ShaderType}
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

		let mesh_data = MeshData::new(&engine, VERTICES.to_vec(), INDICES.to_vec()).unwrap().unwrap();

		let vertex_binary = Shader::compile(&engine, "vertex.glsl.vert", ShaderType::Vertex, VERTEX_SOURCE).unwrap();
		let vertex_shader = Shader::new(&engine, vertex_binary).unwrap().unwrap();

		let fragment_binary = Shader::compile(&engine, "fragment.glsl.frag", ShaderType::Fragment, FRAGMENT_SOURCE).unwrap();
		let fragment_shader = Shader::new(&engine, fragment_binary).unwrap().unwrap();

		let pipeline = Pipeline::new(&engine, &[vertex_shader, fragment_shader]).unwrap().unwrap();

		let render_object = RenderObject::new(&engine, mesh_data, pipeline).unwrap().unwrap();

		drop(render_object);
	}
}
