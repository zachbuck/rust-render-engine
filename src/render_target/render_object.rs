
use uuid::Uuid;
use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer};

use crate::{
	RenderEngine, 
	mesh_data::{MeshData, MeshDataInternal}, 
	render_target::{RenderCall, RenderTarget}, 
	shader::{GraphicsProgram, GraphicsProgramInternal}
};

pub struct RenderObjectHandle {
	uuid: Uuid,	
}

pub(crate) struct RenderObjectInternal {
	surfaces: Vec<Uuid>,
	mesh_data: MeshDataInternal,
	graphics_program: GraphicsProgramInternal,
}

impl RenderObjectInternal {

}

impl RenderCall for RenderObjectInternal {
	fn render_call<'a>(&self, builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>) -> &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer> {
		unsafe { builder
			.bind_vertex_buffers(0, self.mesh_data.vertices.clone()).unwrap()
			.bind_index_buffer(self.mesh_data.indices.clone()).unwrap()
			.bind_pipeline_graphics(self.graphics_program.pipeline.clone()).unwrap()
			.draw_indexed(self.mesh_data.indices.len() as u32, 1, 0, 0, 0).unwrap() 
		}
	}

	fn get_render_surface_uuids(&self) -> Vec<Uuid> {
		return self.surfaces.clone();
	}
}

impl RenderEngine {
	pub fn create_render_object(&mut self, mesh_data: MeshData, graphics_program: GraphicsProgram) -> RenderObjectHandle{
		let uuid = Uuid::now_v7();

		let mesh_data = self.mesh_data.get(&mesh_data.uuid).unwrap();
		let graphics_program = self.graphics_programs.get(&graphics_program.uuid).unwrap();
		
		let internal = RenderObjectInternal {
			surfaces: Vec::new(),
			mesh_data: mesh_data.clone(),
			graphics_program: graphics_program.clone()
		};

		self.render_targets.insert(uuid, RenderTarget::Object(internal));

		RenderObjectHandle {
			uuid: uuid,
		}
	}

	pub fn render_render_object(&mut self, handle: RenderObjectHandle) {
		let render_caller = self.render_targets.get(&handle.uuid).unwrap().to_render_caller();
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