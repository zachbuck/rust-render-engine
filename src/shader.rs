
use std::sync::Arc;

use uuid::Uuid;
use vulkano::pipeline::GraphicsPipeline;

pub struct Shader {
	uuid: Uuid,
}

pub(crate) struct ShaderInternal {
	uuid: Uuid,
}

pub struct GraphicsProgram {
	pub(crate) uuid: Uuid,
}

#[derive(Clone)]
pub(crate) struct GraphicsProgramInternal {
	uuid: Uuid,

	pub(crate) pipeline: Arc<GraphicsPipeline>,
}