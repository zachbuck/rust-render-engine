
use std::{
	marker::PhantomData, 
	sync::{Arc, mpsc::Sender},
};

use uuid::Uuid;

use crate::interface::{
	RenderEngine,
	data_format::{
		index::IndexCollection,
		vertex::{VertexCollection, Vertex2D},
	},
	engine_command::{EngineCommand, MeshDataCommand},
	engine_future::{
		EngineFuture,
		channel_engine_future::ChannelEngineFuture,
		transform_engine_future::TransformEngineFuture,
	},
};

pub struct MeshData<V, I> {
	vertex_format: PhantomData<V>,
	index_length: PhantomData<I>,

	pub(crate) uuid: Uuid,
	command_channel: Sender<EngineCommand>,
}

impl MeshData<Vertex2D, u32> {
	pub fn new(render_engine: &RenderEngine, vertices: Box<[Vertex2D]>, indices: Box<[u32]>) -> impl EngineFuture<Result<Arc<Self>, ()>> {
		let command_channel = render_engine.command_channel.clone();
		let (future, response) = TransformEngineFuture::new(
			ChannelEngineFuture::new(), 
			Box::new(|result: Result<_, _>| result.map(|(uuid,)|
				Arc::new(MeshData { vertex_format: PhantomData, index_length: PhantomData, uuid, command_channel })
			)),
		);

		let command = MeshDataCommand::CreateMeshData {
			vertices: VertexCollection::Vertex2D(vertices),
			indices: IndexCollection::U32(indices),

			response,
		};
		let command = EngineCommand::MeshDataCommand(Box::new(command));
		let _ = render_engine.command_channel.send(command);

		return future;
	}
}

impl<V, I> Drop for MeshData<V, I> {
	fn drop(&mut self) {
		let command = MeshDataCommand::DropMeshData { uuid: self.uuid };
		let command = EngineCommand::MeshDataCommand(Box::new(command));
		let _ = self.command_channel.send(command);
	}
}
