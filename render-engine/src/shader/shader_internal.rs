
use std::collections::HashMap;

use uuid::Uuid;
use vulkano::shader::EntryPoint;

use crate::{
	render_engine::render_thread::RenderThread, 
	shader::{
		ShaderType, 
		descriptor_requirements::DescriptorRequirements
	}
};

#[derive(Debug)]
pub(crate) struct ShaderInternal {
	pub(crate) entry_point: EntryPoint,
	pub(crate) descriptor_requirements: DescriptorRequirements,
}

impl ShaderInternal {
	#[inline]
	pub fn get_shader_type(&self) -> ShaderType { self.entry_point.info().execution_model.into() }
}

impl RenderThread {
	#[inline]
	pub(crate) fn get_shader_internal<'a>(shaders: &'a HashMap<Uuid, ShaderInternal>, uuid: &Uuid) -> Option<&'a ShaderInternal> { 
		shaders.get(uuid)
	}
}

