
use std::{
	collections::HashMap, 
	sync::Weak,
};

use uuid::Uuid;
use vulkano::shader::EntryPoint;

use crate::{
	render_engine::render_thread::RenderThread, 
	shader::{
		Shader, 
		descriptor_requirements::DescriptorRequirements,
	}
};

#[derive(Debug)]
pub(crate) struct ShaderInternal {
	pub(crate) reference: Weak<Shader>,

	pub(crate) entry_point: EntryPoint,
	pub(crate) descriptor_requirements: DescriptorRequirements,
}

impl RenderThread {
	#[inline]
	pub(crate) fn get_shader_internal<'a>(shaders: &'a HashMap<Uuid, ShaderInternal>, uuid: &Uuid) -> Option<&'a ShaderInternal> { 
		shaders.get(uuid)
	}
}

