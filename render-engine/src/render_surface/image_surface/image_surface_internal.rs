
use std::{
	collections::HashMap, 
	sync::Arc,
};

use uuid::Uuid;
use vulkano::{
	command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo, SubpassEndInfo}, 
	device::Queue, 
	format::ClearValue, 
	pipeline::graphics::viewport::{Scissor, Viewport}, 
	render_pass::{Framebuffer, RenderPass}, 
	sync::{
		GpuFuture, 
		future::FenceSignalFuture
	}
};

use crate::{
	macros::error_map, render_engine::{
		render_resources::RenderResources, 
		render_thread::RenderThread
	}, 
	render_surface::RenderSurfaceInternal, 
	renderable::Renderable
};

pub(crate) struct ImageSurfaceInternal {
	pub(crate) render_pass: Arc<RenderPass>,
	pub(crate) framebuffer: Arc<Framebuffer>,
	pub(super) operation_future: Option<Arc<FenceSignalFuture<Box<dyn GpuFuture + Send>>>>,
}

impl RenderSurfaceInternal for ImageSurfaceInternal {
	fn begin_rendering<'a>(&mut self, builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>) -> Result<&'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, bool> {
		if !self.operation_future.as_ref().map(|f| f.is_signaled().unwrap()).unwrap_or(true) { return Err(false) }
		self.operation_future.as_mut().map(|f| f.cleanup_finished());

		let extents = self.framebuffer.extent();

		builder.begin_render_pass(
			RenderPassBeginInfo {
				render_pass: self.render_pass.clone(),
				clear_values: vec![
					Some([0.0, 0.0, 0.0, 1.0].into()),
					Some(ClearValue::Depth(1.0)),
				],
				..RenderPassBeginInfo::framebuffer(self.framebuffer.clone())
			}, 
			SubpassBeginInfo::default()
		).unwrap();

		builder
			.set_scissor_with_count(vec![ Scissor { offset: [0, 0], extent: extents } ].into()).map_err(|_| true)?
			.set_viewport_with_count(vec![Viewport { offset: [0.0, 0.0], extent: [extents[0] as f32, extents[1] as f32], depth_range: 0.0..=1.0 }].into()).map_err(|_| true)?;

		Ok(builder)
	}

	fn render_renderable<'a>(&self, builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, renderable: &Box<dyn Renderable>, resources: &RenderResources) -> Result<&'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, ()> {
		renderable.draw(builder, resources)?;

		Ok(builder)
	}

	fn end_rendering(&mut self, mut builder: AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, mut future: Box<dyn GpuFuture + Send>, queue: Arc<Queue>) -> Result<Box<dyn GpuFuture + Send>, ()> {
		builder.end_render_pass(SubpassEndInfo::default()).map_err(error_map!())?;
		let buffer = builder.build().map_err(error_map!())?;

		if self.operation_future.is_some() {
			future = future
				.join(self.operation_future.as_ref().unwrap().clone()).boxed_send();
		}

		let future = future
			.then_execute(queue.clone(), buffer).map_err(error_map!())?.boxed_send()
			.then_signal_fence_and_flush().map_err(error_map!())?;

		let future = Arc::new(future);
		self.operation_future = Some(future.clone());

		return Ok(future.boxed_send());
	}

	fn as_any(&self) -> &dyn std::any::Any { self }
	fn as_mut_any(&mut self) -> &mut dyn std::any::Any { self }
}

impl RenderThread {
	#[inline]
	#[expect(dead_code)]
	pub(crate) fn get_image_surface<'a>(render_surfaces: &'a HashMap<Uuid, Box<dyn RenderSurfaceInternal>>, uuid: &Uuid) -> Option<&'a ImageSurfaceInternal> {
		Self::get_render_surface(render_surfaces, uuid)?.as_any().downcast_ref()
	}

	#[inline]
	pub(crate) fn get_mut_image_surface<'a>(render_surfaces: &'a mut HashMap<Uuid, Box<dyn RenderSurfaceInternal>>, uuid: &Uuid) -> Option<&'a mut ImageSurfaceInternal> {
		Self::get_mut_render_surface(render_surfaces, uuid)?.as_mut_any().downcast_mut()
	}
}