
use std::{collections::HashMap, sync::Arc};

use uuid::Uuid;
use vulkano::{
	command_buffer::allocator::StandardCommandBufferAllocator, 
	device::{Device, Queue}, 
	memory::allocator::StandardMemoryAllocator, sync::GpuFuture
};

use crate::{
	mesh_data::MeshDataInternal, 
	render_surface::RenderSurface, 
	render_target::RenderTarget, 
	shader::{GraphicsProgramInternal, ShaderInternal}
};

/*	TODO
	- Add error types to Results
 */

pub mod mesh_data;
pub mod render_surface;
pub mod render_target;
pub mod shader;

pub struct RenderEngine {
	mesh_data: HashMap<Uuid, MeshDataInternal>,
	shaders: HashMap<Uuid, ShaderInternal>,
	graphics_programs: HashMap<Uuid, GraphicsProgramInternal>,

	render_surfaces: HashMap<Uuid, RenderSurface>,
	render_targets: HashMap<Uuid, RenderTarget>,

	device: Arc<Device>,
	graphics_queue: Arc<Queue>,
	transfer_queue: Arc<Queue>,
	previous_operation: Option<Box<dyn GpuFuture>>,

	command_allocator: Arc<StandardCommandBufferAllocator>,
	buffer_allocator: Arc<StandardMemoryAllocator>,
}

impl RenderEngine {
	pub fn new() -> Result<RenderEngine, ()> {
		todo!()
	}
}

#[cfg(test)]
mod tests {
    use crate::RenderEngine;

	#[test]
	fn render_engine_new() {
		let _render_engine = RenderEngine::new().unwrap();
	}
}