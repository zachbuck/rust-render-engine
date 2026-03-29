
use std::{
	collections::HashMap, 
	sync::Weak,
};

use uuid::Uuid;
use vulkano::shader::EntryPoint;

use crate::{
	render_engine::render_thread::RenderThread, 
	shader::Shader,
};

#[derive(Debug)]
pub(crate) struct ShaderInternal {
	pub(crate) reference: Weak<Shader>,

	pub(crate) entry_point: EntryPoint,
}

impl RenderThread {
	#[inline]
	pub(crate) fn get_shader_internal<'a>(shaders: &'a HashMap<Uuid, ShaderInternal>, uuid: &Uuid) -> Option<&'a ShaderInternal> { 
		shaders.get(uuid)
	}

	#[inline]
	#[expect(dead_code)]
	pub(crate) fn get_mut_shader_internal<'a>(shaders: &'a mut HashMap<Uuid, ShaderInternal>, uuid: &Uuid) -> Option<&'a mut ShaderInternal> {
		shaders.get_mut(uuid)
	}
}

