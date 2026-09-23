
use uuid::Uuid;

use crate::{
	engine_command::EngineCommand,
	engine_future::channel_engine_future::ChannelEngineResponse,
	surface::{
		RenderPassCreateInfo,
		window_surface::WindowSurfaceCreateInfo,
	}
};

#[derive(Debug)]
pub enum WindowSurfaceCommand {
	CreateWindowSurface {
		create_info: 		WindowSurfaceCreateInfo,
		render_pass_info: 	RenderPassCreateInfo,

		response:			ChannelEngineResponse<Result<(Uuid,), ()>>,
	},

	DropWindowSurface {
		uuid: 				Uuid,
	},
}

impl Into<EngineCommand> for WindowSurfaceCommand {
	fn into(self) -> EngineCommand { EngineCommand::WindowSurfaceCommand(Box::new(self)) }
}
