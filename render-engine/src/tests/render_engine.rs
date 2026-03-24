
use crate::render_engine::{RenderEngine, RenderEngineCreateInfo};

#[test]
/// Ensure that `RenderEngine::new()` and `RenderEngine::drop()` are working as expected.
fn new_render_engine() {
	let create_info = RenderEngineCreateInfo::new()
		.with_app_name("Test".to_string())
		.with_app_vers(10, 10, 10)
		.with_spirv_compiler();
	let engine = RenderEngine::new(create_info).unwrap();

	drop(engine);
}