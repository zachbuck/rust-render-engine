
use std::sync::{
	Arc, 
	mpsc::sync_channel,
};

use uuid::Uuid;
use vulkano::{
	buffer::BufferContents,
	pipeline::graphics::vertex_input::Vertex,
};

use crate::{
	mesh_data::mesh_data_command::MeshDataCommand, 
	render_engine::{
		RenderEngine, 
		engine_future::EngineFuture,
	},
};

pub(crate) mod mesh_data_command;
pub(crate) mod mesh_data_internal;

#[derive(Debug)]
pub struct MeshData {
	pub(crate) uuid: Uuid,
	render_engine: Arc<RenderEngine>,
}

#[repr(C)]
#[derive(BufferContents, Vertex)]
#[derive(Clone, Copy)]
#[derive(Debug)]
pub struct Vertex3D {
	#[format(R32G32B32_SFLOAT)]
	pub position: [f32; 3],
	#[format(R32G32B32_SFLOAT)]
	pub normal: [f32; 3],
	#[format(R32G32_SFLOAT)]
	pub uv: [f32; 2],
}

impl MeshData {
	pub fn new(render_engine: &Arc<RenderEngine>, vertices: Vec<Vertex3D>, indices: Vec<u16>) -> EngineFuture<Result<Arc<Self>, ()>> {
		let (send, recv) = sync_channel(1);

		render_engine.command_channel.send(
			MeshDataCommand::CreateMeshData {
				sender: send,

				vertices: vertices.into_boxed_slice(),
				indices: indices.into_boxed_slice(),
				engine: render_engine.clone(),
			}.into()
		).unwrap();

		return EngineFuture::new_single(recv);
	}

	pub fn get_all(render_engine: &Arc<RenderEngine>) -> EngineFuture<Result<Box<[Arc<MeshData>]>, ()>> {
		let (send, recv) = sync_channel(1);

		render_engine.command_channel.send(
			MeshDataCommand::GetMeshData { 
				sender: send
			}.into()
		).unwrap();

		EngineFuture::new_single(recv)
	}
}

impl Drop for MeshData {
	fn drop(&mut self) {
		self.render_engine.command_channel.send(
			MeshDataCommand::DropMeshData { 
				uuid: self.uuid 
			}.into()
		).unwrap();
	}
}

#[cfg(test)]
/// Tests for public facing API of `MeshData`
mod tests {
    use std::sync::Arc;

    use crate::{
		mesh_data::{MeshData, Vertex3D}, 
		render_engine::{RenderEngine, RenderEngineCreateInfo}
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

	#[test]
	/// Ensure `MeshData::new()` and `MeshData::drop()` are working as expected.
	fn new_mesh_data() {
		let create_info = RenderEngineCreateInfo::new();
		let engine = RenderEngine::new(create_info).unwrap();

		let mesh_data = MeshData::new(&engine, VERTICES.to_vec(), INDICES.to_vec()).unwrap().unwrap();

		drop(mesh_data);

		let mesh_data_list = MeshData::get_all(&engine).unwrap().unwrap();
		assert!(mesh_data_list.len() == 0);
	}

	#[test]
	/// Ensure `MeshData::get_all()` is working as expected.
	fn get_all() {
		let create_info = RenderEngineCreateInfo::new();
		let engine = RenderEngine::new(create_info).unwrap();

		let mesh_data = MeshData::new(&engine, VERTICES.to_vec(), INDICES.to_vec()).unwrap().unwrap();

		let mesh_data_list = MeshData::get_all(&engine).unwrap().unwrap();

		assert!(mesh_data_list.len() == 1);
		assert!(Arc::ptr_eq(&mesh_data, &mesh_data_list[0]));
	}
}
