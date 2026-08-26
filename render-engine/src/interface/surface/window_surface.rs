
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
	surface::{RenderPassCreateInfo, Surface, SurfaceInfo},
};

pub struct WindowSurface {
	uuid: 				Uuid,
	command_channel: 	Sender<EngineCommand>,
}

#[derive(Debug)]
pub struct WindowSurfaceCreateInfo {
	pub title: 			String,
	pub dimensions: 	[u32; 2],
	pub clear_color: 	[f32; 4],
}

impl WindowSurface {
	pub fn new(render_engine: &Arc<RenderEngine>, create_info: WindowSurfaceCreateInfo, render_pass_info: RenderPassCreateInfo) -> impl EngineFuture<Result<Arc<WindowSurface>, ()>> {
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

		let _ = render_engine.command_channel.send(WindowSurfaceCommand::CreateWindowSurface { 
			create_info: 		create_info,
			render_pass_info: 	render_pass_info,
			response: 			response, 
		}.into());

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

impl Default for WindowSurfaceCreateInfo {
	fn default() -> Self {
		WindowSurfaceCreateInfo {
			title:			"My Window".to_string(),
			dimensions:		[800, 600],
			clear_color: 	[0.0, 0.0, 0.0, 1.0],
		}
	}
}
