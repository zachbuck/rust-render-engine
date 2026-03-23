
use std::sync::Arc;

use vulkano::{
	command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer}, 
	pipeline::GraphicsPipeline
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
