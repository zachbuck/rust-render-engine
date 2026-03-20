
use std::sync::Arc;

use vulkano::pipeline::GraphicsPipeline;

use crate::{
	pipeline::Pipeline, 
	render_engine::render_thread::RenderThread, 
	shader::Shader
};

#[derive(Debug)]
pub(crate) struct PipelineInternal {
	pub(crate) pipeline: Arc<GraphicsPipeline>,
	pub(crate) shaders: Box<[Arc<Shader>]>,
}

impl RenderThread {
	#[inline]
	#[expect(dead_code)]
	pub(crate) fn get_pipeline_internal(&self, reference: Arc<Pipeline>) -> Option<&PipelineInternal> { self.pipelines.get(&reference.uuid) }

	#[inline]
	#[expect(dead_code)]
	pub(crate) fn get_mut_pipeline_internal(&mut self, reference: Arc<Pipeline>) -> Option<&mut PipelineInternal> { self.pipelines.get_mut(&reference.uuid) }
}
