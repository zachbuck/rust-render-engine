
use std::{
	thread::sleep, 
	time::Duration,
};

use crate::render_engine::{RenderEngine, RenderEngineCreateInfo};

#[test]
fn new_render_engine() {
	let create_info = RenderEngineCreateInfo::new()
		.with_app_name("Test".to_string())
		.with_app_vers(10, 10, 10)
		.with_spirv_compiler();
	let _engine = RenderEngine::new(create_info).unwrap();
}

#[test]
fn drop_render_engine() {
	let create_info = RenderEngineCreateInfo::new();
	let engine = RenderEngine::new(create_info).unwrap();

	drop(engine);

	sleep(Duration::from_secs(1));
}