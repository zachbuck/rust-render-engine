
use std::sync::{Arc, Weak};

use vulkano::{
	command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer}, 
	pipeline::GraphicsPipeline
};

use crate::pipeline::Pipeline;

#[derive(Debug)]
pub(crate) struct PipelineInternal {
	pub(crate) reference: Weak<Pipeline>,

	pub(crate) pipeline: Arc<GraphicsPipeline>,
}

impl PipelineInternal {
	pub(crate) fn bind<'a>(&self, builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>) -> Result<&'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, ()> {
		builder
			.bind_pipeline_graphics(self.pipeline.clone()).map_err(|_| ())?;

		return Ok(builder);
	}
}
