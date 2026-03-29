
use std::{collections::HashMap, sync::{Arc, Weak}};

use uuid::Uuid;
use vulkano::{
	command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer}, 
	descriptor_set::layout::DescriptorSetLayout, 
	pipeline::GraphicsPipeline
};

use crate::{
	macros::error_map, pipeline::Pipeline, render_engine::render_thread::RenderThread
};

#[derive(Debug)]
pub(crate) struct PipelineInternal {
	pub(crate) reference: Weak<Pipeline>,

	pub(crate) pipeline: Arc<GraphicsPipeline>,
	pub(crate) descriptor_layouts: HashMap<u32, Arc<DescriptorSetLayout>>,
}

impl PipelineInternal {
	pub(crate) fn bind<'a>(&self, builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>) -> Result<&'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, ()> {
		builder
			.bind_pipeline_graphics(self.pipeline.clone()).map_err(error_map!())?;

		return Ok(builder);
	}
}

impl RenderThread {
	#[inline]
	pub(crate) fn get_pipeline<'a>(pipelines: &'a HashMap<Uuid, PipelineInternal>, uuid: &Uuid, ) -> Option<&'a PipelineInternal> {
		pipelines.get(uuid)
	}

	#[inline]
	#[expect(dead_code)]
	pub(crate) fn get_mut_pipeline<'a>(pipelines: &'a mut HashMap<Uuid, PipelineInternal>, uuid: &Uuid) -> Option<&'a mut PipelineInternal> {
		pipelines.get_mut(uuid)
	}
}
