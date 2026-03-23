
use std::{
	sync::Arc, thread::sleep, time::Duration
};

use crate::{
	mesh_data::{MeshData, Vertex3D}, 
	render_engine::{RenderEngine, RenderEngineCreateInfo}
};

const VERTICES: [Vertex3D; 4] = [
	Vertex3D { position: [ 0.5, 0.5, 0.5], normal: [0.0; 3], uv: [0.0; 2] }, // bottom right
	Vertex3D { position: [-0.5, 0.5, 0.5], normal: [0.0; 3], uv: [0.0; 2] }, // bottom left
	Vertex3D { position: [-0.5,-0.5, 0.5], normal: [0.0; 3], uv: [0.0; 2] }, // top left
	Vertex3D { position: [ 0.5,-0.5, 0.5], normal: [0.0; 3], uv: [0.0; 2] }  // top right
];

const INDICES: [u16; 6] = [
	0, 2, 1,
	0, 3, 2
];

#[test]
fn new_mesh_data() {
	let create_info = RenderEngineCreateInfo::new();
	let engine = RenderEngine::new(create_info).unwrap();

	let _mesh_data = MeshData::new(engine.clone(), VERTICES.to_vec(), INDICES.to_vec()).unwrap().unwrap();
}

#[test]
fn get_all() {
	let create_info = RenderEngineCreateInfo::new();
	let engine = RenderEngine::new(create_info).unwrap();

	let mesh_data = MeshData::new(engine.clone(), VERTICES.to_vec(), INDICES.to_vec()).unwrap().unwrap();

	let mesh_data_list = MeshData::get_all(engine.clone()).unwrap().unwrap();

	assert!(mesh_data_list.len() == 1);
	assert!(Arc::ptr_eq(&mesh_data, &mesh_data_list[0]));
}

#[test]
fn drop_mesh_data() {
	let create_info = RenderEngineCreateInfo::new();
	let engine = RenderEngine::new(create_info).unwrap();

	let mesh_data = MeshData::new(engine.clone(), VERTICES.to_vec(), INDICES.to_vec()).unwrap().unwrap();

	drop(mesh_data);

	sleep(Duration::from_secs(1));

	let mesh_data_list = MeshData::get_all(engine).unwrap().unwrap();
	
	assert!(mesh_data_list.len() == 0);
}