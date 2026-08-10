
use std::sync::{
	Arc, 
	mpsc::Sender,
};

use uuid::Uuid;

use crate::{
	engine_command::{EngineCommand, WindowSurfaceCommand}, 
	engine_future::{
		EngineFuture, 
		channel_engine_future::ChannelEngineFuture, 
		then_transform_future::ThenTransformFuture,
	}, 
	render_engine::RenderEngine, 
	surface::{Surface, SurfaceInfo},
};

pub struct WindowSurface {
	uuid: 				Uuid,
	command_channel: 	Sender<EngineCommand>,
}

impl WindowSurface {
	pub fn new(render_engine: &Arc<RenderEngine>, title: String, dimensions: (u32, u32)) -> impl EngineFuture<Result<Arc<WindowSurface>, ()>> {
		let command_channel = render_engine.command_channel.clone();
		let (future, response) = ThenTransformFuture::new(
			ChannelEngineFuture::new(), 
			Box::new(|result: Result<_, _>| {
				result.map(|(uuid, )| 
					Arc::new(WindowSurface {
						uuid: uuid,
						command_channel: command_channel,
					})
				)
			}),
		);

		render_engine.command_channel.send(WindowSurfaceCommand::CreateWindowSurface { 
			title:		title, 
			dimensions:	dimensions, 
			response: 	response, 
		}.into()).unwrap();

		return future
	}
}

impl Drop for WindowSurface {
	fn drop(&mut self) {
		let _ = self.command_channel.send(WindowSurfaceCommand::DropWindowSurface { uuid: self.uuid }.into());
	}
}

impl Surface for WindowSurface {}

impl SurfaceInfo for WindowSurface {
	fn get_uuid(&self) -> &Uuid { &self.uuid }
}
