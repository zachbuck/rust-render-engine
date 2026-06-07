
use std::sync::{
	Arc, 
	mpsc::Sender,
};

use uuid::Uuid;

use crate::interface::{
	RenderEngine,
	engine_command::{EngineCommand, RenderObjectCommand},
	engine_future::{
		EngineFuture,
		channel_engine_future::ChannelEngineFuture,
		transform_engine_future::TransformEngineFuture,
	},
	mesh_data::MeshData, 
	pipeline::GraphicsPipeline,
	render_target::{RenderTarget, RenderTargetInfo},
};

pub struct RenderObject<V, I> {
	pub mesh: Arc<MeshData<V, I>>,
	pub pipeline: Arc<GraphicsPipeline>,

	pub(crate) uuid: Uuid,
	command_channel: Sender<EngineCommand>,
}

impl<V, I> RenderObject<V, I> 
where V: 'static, I: 'static {
	pub fn new(render_engine: &RenderEngine, mesh: Arc<MeshData<V, I>>, pipeline: Arc<GraphicsPipeline>) -> impl EngineFuture<Result<Arc<Self>, ()>> {
		let command_channel = render_engine.command_channel.clone();
		let mesh_uuid = mesh.uuid;
		let pipeline_uuid = pipeline.uuid;
		let (future, response) = TransformEngineFuture::new(
			ChannelEngineFuture::new(), 
			Box::new(|result: Result<_, _>| result.map(|(uuid,)| {
				Arc::new(RenderObject { mesh: mesh, pipeline: pipeline, uuid, command_channel })
			}))
		);

		let command = RenderObjectCommand::CreateRenderObject { mesh_data: mesh_uuid, pipeline: pipeline_uuid, response };
		let command = EngineCommand::RenderObjectCommand(Box::new(command));
		let _ = render_engine.command_channel.send(command);

		return future
	}
}

impl<V, I> Drop for RenderObject<V, I> {
	fn drop(&mut self) {
		let command = RenderObjectCommand::DropRenderObject { uuid: self.uuid };
		let command = EngineCommand::RenderObjectCommand(Box::new(command));
		let _ = self.command_channel.send(command);
	}
}

impl<V, I> RenderTarget for RenderObject<V, I> {}

impl<V, I> RenderTargetInfo for RenderObject<V, I> {
	fn get_uuid(&self) -> &Uuid { &self.uuid }
	fn get_pipeline(&self) -> &Arc<GraphicsPipeline> { &self.pipeline }
}
