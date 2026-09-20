
use std::sync::{
	Arc, 
	mpsc::Sender,
};

use uuid::Uuid;

use crate::{
	engine_command::{EngineCommand, RenderObjectCommand}, engine_future::{
		EngineFuture, 
		channel_engine_future::ChannelEngineFuture, 
		then_transform_future::ThenTransformFuture,
	}, mesh_data::MeshData, pipeline::Pipeline, render_engine::RenderEngine,
};

pub struct RenderObject {
	pub(crate) uuid: Uuid,
	command_channel: Sender<EngineCommand>,

	pub mesh_data: Arc<MeshData>,
	pub pipeline: Arc<Pipeline>,
}

impl RenderObject {
	// TODO
	// - Check mesh compatibility with pipeline input
	pub fn new(render_engine: &Arc<RenderEngine>, mesh_data: Arc<MeshData>, pipeline: Arc<Pipeline>) -> impl EngineFuture<Result<Arc<Self>, ()>> {
		let command_channel = render_engine.command_channel.clone();
		
		let mesh_uuid = mesh_data.uuid;
		let pipeline_uuid = pipeline.uuid;

		let (future, response) = ThenTransformFuture::new(
			ChannelEngineFuture::new(), 
			Box::new(|result: Result<_, _>| result.map(|(uuid,)| {
				Arc::new(RenderObject {
					uuid: uuid,
					command_channel: command_channel,
					mesh_data: mesh_data,
					pipeline: pipeline,
				})
			})),
		);

		let _ = render_engine.command_channel.send(RenderObjectCommand::CreateRenderObject { 
			mesh_data: mesh_uuid, 
			pipeline: pipeline_uuid, 
			response,
		}.into());

		future
	}
}

impl Drop for RenderObject {
	fn drop(&mut self) {
		let _ = self.command_channel.send(RenderObjectCommand::DropRenderObject { uuid: self.uuid }.into());
	}
}
