
use std::{
	any::Any, 
	collections::HashMap,
};

use uuid::Uuid;
use vulkano::command_buffer::{
	AutoCommandBufferBuilder, 
	PrimaryAutoCommandBuffer,
};

use crate::render_engine::{
	render_resources::RenderResources, 
	render_thread::RenderThread,
};

pub mod descriptor_set_data;
pub mod render_object;

pub(crate) trait Renderable: Any {
	fn draw<'a>(&self, builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, resources: &RenderResources) -> Result<&'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, ()>;

	fn as_any(&self) -> &dyn Any;
	fn as_mut_any(&mut self) -> &mut dyn Any;
}

impl RenderThread {
	#[inline]
	fn get_renderable<'a>(renderables: &'a HashMap<Uuid, Box<dyn Renderable>>, uuid: &Uuid) -> Option<&'a dyn Renderable> {
		renderables.get(uuid).map(|b| b.as_ref())
	}

	#[inline]
	fn get_mut_renderable<'a>(renderables: &'a mut HashMap<Uuid, Box<dyn Renderable>>, uuid: &Uuid) -> Option<&'a mut dyn Renderable> {
		renderables.get_mut(uuid).map(|b| b.as_mut())
	}
}
