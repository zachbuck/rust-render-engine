
use std::{
	marker::PhantomData, 
	sync::{
		Arc, 
		mpsc::Sender,
	},
};

use uuid::Uuid;

use crate::{
	data_formats::Vertex3D, 
	engine_command::{EngineCommand, MeshDataCommand}, 
	engine_future::{
		EngineFuture, 
		channel_engine_future::ChannelEngineFuture, 
		then_transform_future::ThenTransformFuture,
	}, 
	render_engine::RenderEngine,
};

pub struct MeshData {
	uuid: Uuid,
	command_channel: Sender<EngineCommand>,

	pub vertex_format: 	PhantomData<Vertex3D>,
	pub index_format: 	PhantomData<u32>,
}

impl MeshData {
	pub fn new(render_engine: &Arc<RenderEngine>, vertices: Box<[Vertex3D]>, indices: Box<[u32]>) -> impl EngineFuture<Result<Arc<MeshData>, ()>> {
		let command_channel = render_engine.command_channel.clone();
		let (future, response) = ThenTransformFuture::new(
			ChannelEngineFuture::new(), 
			Box::new(|result: Result<_, _>| {
				result.map(|(uuid,)| {
					Arc::new(MeshData {
						uuid: 				uuid,
						command_channel: 	command_channel,
						vertex_format: 		PhantomData,
						index_format: 		PhantomData,
					})	
				})
			})
		);

		let _ = render_engine.command_channel.send(MeshDataCommand::CreateMeshData {
			vertices: vertices,
			indices: indices,
			response: response,
		}.into());

		return future;
	}
}

impl Drop for MeshData {
	fn drop(&mut self) {
		let _ = self.command_channel.send(MeshDataCommand::DropMeshData { uuid: self.uuid }.into());
	}
}
