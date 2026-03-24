
use std::sync::mpsc::SyncSender;

use uuid::Uuid;
use vulkano::{
	command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage}, 
	sync::{self, GpuFuture}
};

use crate::render_engine::{
	render_command::RenderEngineCommand, 
	render_thread::RenderThread
};

#[derive(Debug)]
pub(crate) enum RenderSurfaceCommand {
	RenderRenderSurface {
		sender: SyncSender<Result<(), ()>>,

		uuid: Uuid
	}
}

impl Into<RenderEngineCommand> for RenderSurfaceCommand {
	fn into(self) -> RenderEngineCommand {
		RenderEngineCommand::RenderSurfaceCommand(self)
	}
}

impl RenderThread {
	pub(crate) fn process_render_surface_command(&mut self, command: RenderSurfaceCommand) {
		match command {
			RenderSurfaceCommand::RenderRenderSurface { sender, uuid } => { let _ = sender.send(self.render_render_surface(uuid)); },
		}
	}

	fn render_render_surface(&mut self, uuid: Uuid) -> Result<(), ()> {
		let queue = self.get_graphics_queue();

		let render_surface = self.get_render_surface(&uuid).ok_or(())?;

		let mut builder = AutoCommandBufferBuilder::primary(
			self.command_allocator.clone(), 
			queue.queue_family_index(), 
			CommandBufferUsage::OneTimeSubmit,
		).map_err(|_| ())?;

		let render_resources = self.generate_render_resources();

		render_surface.begin_rendering(&mut builder)?;

		for (_, renderable) in &self.renderables {
			render_surface.render_renderable(&mut builder, renderable, &render_resources)?;
		}

		let future = self.graphics_future.take().unwrap();

		let render_surface = self.get_mut_render_surface(&uuid).ok_or(())?;

		let result = render_surface.end_rendering(builder, future, queue);

		if result.is_err() {
			self.graphics_future = Some(sync::now(self.device.clone()).boxed_send());
			
			return Err(())
		} else {
			let new_future = result.unwrap();
			self.graphics_future = Some(new_future);

			return Ok(())
		}
	}
}
