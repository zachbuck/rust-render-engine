
use uuid::Uuid;
use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer};

pub mod render_object;
pub use render_object::RenderObjectHandle as RenderObject;

use crate::{
	RenderEngine, 
	render_target::render_object::RenderObjectInternal
};

pub(crate) enum RenderTarget {
	Object(RenderObjectInternal)
}

impl RenderTarget {
	pub(crate) fn to_render_caller(&self) -> &dyn RenderCall {
		match self {
			Self::Object(render_object) => render_object,
		}
	}
}

pub(crate) trait RenderCall {
	fn render_call<'a>(&self, builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>) -> &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>;

	fn get_render_surface_uuids(&self) -> Vec<Uuid>;
}

impl RenderEngine {
	pub fn render_all_render_targets(&mut self) {
		for (_, render_target) in &self.render_targets {
			let render_caller = render_target.to_render_caller();
			let uuids = render_caller.get_render_surface_uuids();
			if uuids.len() == 0 { 
				for (_, render_surface) in &mut self.render_surfaces {
					render_surface.process_render_queue(render_caller);
				}
			} else {
				for uuid in &uuids {
					self.render_surfaces.get_mut(uuid).unwrap().process_render_queue(render_caller);
				}
			}
		}
	}
}