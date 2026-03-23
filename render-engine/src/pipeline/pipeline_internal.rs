
use std::sync::Arc;

use vulkano::{
	command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer}, 
	pipeline::GraphicsPipeline
};

use crate::{
	pipeline::Pipeline, 
	render_engine::render_thread::RenderThread
};

#[derive(Debug)]
pub(crate) struct PipelineInternal {
	pub(crate) pipeline: Arc<GraphicsPipeline>,
}

impl PipelineInternal {
	pub(crate) fn bind<'a>(&self, builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>) -> Result<&'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, ()> {
		builder
			.bind_pipeline_graphics(self.pipeline.clone()).map_err(|_| ())?;

		return Ok(builder);
	}
}

impl RenderThread {
	#[inline]
	#[expect(dead_code)]
	pub(crate) fn get_pipeline_internal(&self, reference: Arc<Pipeline>) -> Option<&PipelineInternal> { self.pipelines.get(&reference.uuid) }

	#[inline]
	#[expect(dead_code)]
	pub(crate) fn get_mut_pipeline_internal(&mut self, reference: Arc<Pipeline>) -> Option<&mut PipelineInternal> { self.pipelines.get_mut(&reference.uuid) }
}
