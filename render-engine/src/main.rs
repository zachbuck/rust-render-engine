
use std::{
	thread, 
	time::Duration,
};

use render_engine::{
	engine_future::EngineFuture, 
	render_engine::{RenderEngine, RenderEngineBackend, RenderEngineCreateInfo}, 
	render_instruction_buffer::RenderInstructionBufferBuilder, 
	surface::window_surface::WindowSurface,
};

fn main() -> () {
	let render_engine = RenderEngine::new(RenderEngineCreateInfo::with_backend(RenderEngineBackend::Vulkan)).unwrap();
	
	let window = WindowSurface::new(&render_engine, "New Window".to_string(), (800, 600)).wait().unwrap();

	let builder = RenderInstructionBufferBuilder::begin(&window);

	let instruction_buffer = builder.build();

	render_engine.submit_render_instructions(instruction_buffer).wait().unwrap();

	thread::sleep(Duration::from_secs(5));
}
