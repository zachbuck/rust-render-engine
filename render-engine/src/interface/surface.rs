
use std::sync::{
	Arc, 
	mpsc::Sender,
};

use uuid::Uuid;

use crate::engine_command::EngineCommand;

pub mod image_surface;

#[allow(private_bounds)]
pub trait Surface: SurfaceInfo {}

pub(crate) trait SurfaceInfo {
	fn get_command_channel(&self) -> &Sender<EngineCommand>;
	fn get_uuid(&self) -> &Uuid;
}

impl<T> Surface for Arc<T>
where T: Surface {}

impl<T> SurfaceInfo for Arc<T>
where T: Surface {
	fn get_command_channel(&self) -> &Sender<EngineCommand> { self.as_ref().get_command_channel() }
	fn get_uuid(&self) -> &Uuid { self.as_ref().get_uuid() }
}
