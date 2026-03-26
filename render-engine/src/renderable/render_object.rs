
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
