
use std::sync::Arc;

use vulkano::shader::EntryPoint;

use crate::{
	render_engine::render_thread::RenderThread, 
	shader::{Shader, ShaderType}
};

#[derive(Debug)]
pub(crate) struct ShaderInternal {
	pub(crate) entry_point: EntryPoint,
}

impl ShaderInternal {
	#[inline]
	pub fn get_shader_type(&self) -> ShaderType { self.entry_point.info().execution_model.into() }
}

impl RenderThread {
	#[inline]
	pub(crate) fn get_shader_internal(&self, reference: Arc<Shader>) -> Option<&ShaderInternal> { self.shaders.get(&reference.uuid) }
	
	#[inline]
	#[expect(dead_code)]
	pub(crate) fn get_mut_shader_internal(&mut self, reference: Arc<Shader>) -> Option<&mut ShaderInternal> { self.shaders.get_mut(&reference.uuid) }
}

