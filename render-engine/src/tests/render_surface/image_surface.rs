use std::{thread::sleep, time::Duration};

use crate::{render_engine::{RenderEngine, RenderEngineCreateInfo}, render_surface::image_surface::ImageSurface};


#[test]
fn new_image_surface() {
	let create_info = RenderEngineCreateInfo::new();
	let engine = RenderEngine::new(create_info).unwrap();

	let _image_surface = ImageSurface::new(engine.clone(), 100, 100).unwrap().unwrap();
}

#[test]
fn drop_image_surface() {
	let create_info = RenderEngineCreateInfo::new();
	let engine = RenderEngine::new(create_info).unwrap();

	let image_surface = ImageSurface::new(engine.clone(), 100, 100).unwrap().unwrap();

	drop(image_surface);

	sleep(Duration::from_secs(1));
}

#[test]
fn get_image_surface_data() {
	let create_info = RenderEngineCreateInfo::new();
	let engine = RenderEngine::new(create_info).unwrap();

	let image_surface = ImageSurface::new(engine.clone(), 100, 100).unwrap().unwrap();

	image_surface.render_all().unwrap().unwrap();

	let data = image_surface.get_image_surface_data().unwrap().unwrap();
	assert!(data.len() == 100 * 100 * 4);
	for x in 0..100 * 100 * 4 {
		if x % 4 == 3 {
			assert!(data[x] == 255) // alpha component
		} else {
			assert!(data[x] == 0) // color component
		}
	}
}
