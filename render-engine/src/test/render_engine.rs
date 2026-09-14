use crate::render_engine::{RenderEngine, RenderEngineBackend, RenderEngineCreateInfo};


#[test]
fn new_render_engine() {
	let result = RenderEngine::new(RenderEngineCreateInfo::with_backend(RenderEngineBackend::Vulkan));

	assert!(result.is_ok(), "RenderEngine failed to be created");
}
