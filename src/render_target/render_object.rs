
use uuid::Uuid;

use vulkano::{
	command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer}, 
	pipeline::Pipeline
};

use crate::{
	RenderEngine, 
	mesh_data::{MeshData, MeshDataInternal}, 
	render_target::{RenderCall, RenderTarget}, 
	shader::{
		GraphicsProgram, 
		GraphicsProgramInternal, 
		descriptor::{Descriptor, DescriptorData}
	}, 
	unwrap_option_or_none
};

pub struct RenderObjectHandle {
	uuid: Uuid,	
}

pub(crate) struct RenderObjectInternal {
	surfaces: Vec<Uuid>,
	mesh_data: MeshDataInternal,
	graphics_program: GraphicsProgramInternal,

	descriptors: Vec<(Descriptor, DescriptorData)>,
}

impl RenderCall for RenderObjectInternal {
	fn render_call<'a>(&self, builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>) -> &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer> {
		builder
			.bind_vertex_buffers(0, self.mesh_data.vertices.clone()).unwrap()
			.bind_index_buffer(self.mesh_data.indices.clone()).unwrap()
			.bind_pipeline_graphics(self.graphics_program.pipeline.clone()).unwrap();
		
		let sets = self.descriptors.chunk_by(|(a, _), (b, _)| a.set == b.set);
		for set_descriptors in sets {
			let mut set = 0;
			let mut set_writes = Vec::with_capacity(set_descriptors.len());
			for (descriptor, data) in set_descriptors {
				set = descriptor.set;
				set_writes.push(data.to_descriptor_write(descriptor.binding));
			}
			
			builder.push_descriptor_set(
				vulkano::pipeline::PipelineBindPoint::Graphics, 
				self.graphics_program.pipeline.layout().clone(), 
				set, 
				set_writes.into()
			).unwrap();
		}

		unsafe { builder
			.draw_indexed(self.mesh_data.indices.len() as u32, 1, 0, 0, 0).unwrap()
		}
	}

	fn get_render_surface_uuids(&self) -> Vec<Uuid> {
		return self.surfaces.clone();
	}
}

impl RenderEngine {
	pub fn create_render_object(&mut self, mesh_data: MeshData, graphics_program: GraphicsProgram) -> Result<RenderObjectHandle, ()> {
		let uuid = Uuid::now_v7();

		let mesh_data = unwrap_option_or_none!(self.mesh_data.get(&mesh_data.uuid));
		let graphics_program = unwrap_option_or_none!(self.graphics_programs.get(&graphics_program.uuid));

		let mut descriptors = Vec::new();
		for descriptor in &graphics_program.descriptors {
			descriptors.push((descriptor.clone(), DescriptorData::from_descriptor(descriptor, self.buffer_allocator.clone())))
		}

		let internal = RenderObjectInternal {
			surfaces: Vec::new(),
			mesh_data: mesh_data.clone(),
			graphics_program: graphics_program.clone(),
			descriptors: descriptors
		};

		self.render_targets.insert(uuid, RenderTarget::Object(internal));

		Ok(RenderObjectHandle {
			uuid: uuid,
		})
	}

	pub fn render_render_object(&mut self, handle: RenderObjectHandle) -> Result<(), ()> {
		let render_caller = unwrap_option_or_none!(self.render_targets.get(&handle.uuid)).to_render_caller();
		let uuids = render_caller.get_render_surface_uuids();
		if uuids.len() == 0 {
			for (_, render_surface) in &mut self.render_surfaces {
				render_surface.process_render_queue(render_caller);
			}
		} else {
			for uuid in &uuids {
				unwrap_option_or_none!(self.render_surfaces.get_mut(uuid)).process_render_queue(render_caller);
			}
		}

		Ok(())
	}
}