
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
		engine_future::EngineFuture
	}, 
	renderable::render_object::render_object_command::RenderObjectCommand
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
