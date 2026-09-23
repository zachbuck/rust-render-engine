
use std::collections::HashMap;

use crate::macros::{debug_error, debug_none};

#[derive(Debug)]
pub struct RawObjVertex {
	pub position: 	[f32; 4],
	pub texture:	Option<[f32; 3]>,
	pub normal:		Option<[f32; 3]>,
}

#[derive(Debug)]
pub struct ObjMesh<V> where
V: From<RawObjVertex> {
	name: 		String,
	vertices: 	Box<[V]>,
	indices: 	Box<[u32]>,

	groups:		Box<[(usize, usize, String)]>,
}

pub struct ObjMeshGroup<'a, V> where 
V: From<RawObjVertex> {
	pub name: &'a str,
	pub vertices: &'a [V],
	pub indices: &'a [u32],
}

impl<V> ObjMesh<V> where 
V: From<RawObjVertex> {
	pub fn get_name(&self) -> &str { &self.name }
	pub fn get_vertices(&self) -> &[V] { &self.vertices }
	pub fn get_indices(&self) -> &[u32] { &self.indices }

	pub fn get_groups(& self) -> Box<[ObjMeshGroup<'_, V>]> { 
		self.groups.iter().map(|(start, end, name)| {
			ObjMeshGroup {
				name: name,
				vertices: &self.vertices,
				indices: &self.indices[*start..*end],
			}
		}).collect::<Box<[_]>>()
	}

	pub fn parse(obj_file: &str) -> Result<Box<[Self]>, ()> {
		const UNDERSTOOD_PREFIXES: [&str; 6] = ["v ", "vt ", "vn ", "o ", "g ", "f "];

		let mut position_data = Vec::new();
		let mut texture_data = Vec::new();
		let mut normal_data = Vec::new();

		let mut objects = Vec::new();

		for line in obj_file.lines().map(|str| str.trim()) {
			if !UNDERSTOOD_PREFIXES.iter().any(|pre| line.starts_with(*pre)) { continue; }
			
			if line.starts_with("v ") {
				let line = line[2..].split(" ")
					.map(|str| str.parse::<f32>())
					.collect::<Vec<_>>();

				position_data.push([
					*line.get(0).ok_or_else(debug_none!())?.as_ref().map_err(debug_error!())?,
					*line.get(1).ok_or_else(debug_none!())?.as_ref().map_err(debug_error!())?,
					*line.get(2).ok_or_else(debug_none!())?.as_ref().map_err(debug_error!())?,
					*line.get(3).unwrap_or(&Ok(1.0)).as_ref().map_err(debug_error!())?,
				]);
				continue;
			} else if line.starts_with("vt ") {
				let line = line[3..].split(" ")
					.map(|str| str.parse::<f32>())
					.collect::<Vec<_>>();

				texture_data.push([
					*line.get(0).ok_or_else(debug_none!())?.as_ref().map_err(debug_error!())?,
					*line.get(1).unwrap_or(&Ok(0.0)).as_ref().map_err(debug_error!())?,
					*line.get(2).unwrap_or(&Ok(0.0)).as_ref().map_err(debug_error!())?,
				]);
				continue;
			} else if line.starts_with("vn ") {
				let line = line[3..].split(" ")
					.map(|str| str.parse::<f32>())
					.collect::<Vec<_>>();

				normal_data.push([
					*line.get(0).ok_or_else(debug_none!())?.as_ref().map_err(debug_error!())?,
					*line.get(1).ok_or_else(debug_none!())?.as_ref().map_err(debug_error!())?,
					*line.get(2).ok_or_else(debug_none!())?.as_ref().map_err(debug_error!())?,
				]);
				continue;
			} else if line.starts_with("o ") {
				objects.push((line[2..].to_string(), Vec::new(), HashMap::new(), Vec::new()));
				continue;
			}

			if objects.is_empty() {
				objects.push(("".to_string(), Vec::new(), HashMap::new(), Vec::new()));
			}
			let (_, groups, vertex_set, vertex_list) = objects.last_mut().unwrap();

			if line.starts_with("g ") {
				groups.push((line[2..].to_string(), Vec::new()));
			}

			if groups.is_empty() {
				groups.push(("".to_string(), Vec::new()));
			}
			let (_, group) = groups.last_mut().unwrap();

			if line.starts_with("f ") {
				let mut vertices = line[2..].split(" ")
					.map(|str| {
						str.split("/")
							.map(|str| str.parse::<i64>())
							.zip(vec![position_data.len(), texture_data.len(), normal_data.len()])
							.map(|(i, len)| {
								i.map(|i| if i.is_negative() { ((len as i64) + i) as usize } else { i as usize })
							}).collect::<Box<[_]>>()
					}).collect::<Vec<_>>();

				while vertices.len() > 2 {
					macro_rules! extract_vertex {
						($vertex: expr) => {
							{
								let vertex = (
									*$vertex.get(0).ok_or_else(debug_none!())?.as_ref().map_err(debug_error!())?,
									$vertex.get(1).map_or(None, |r| r.as_ref().ok()).map(|i| *i),
									$vertex.get(2).map_or(None, |r| r.as_ref().ok()).map(|i| *i),
								);

								*vertex_set.entry(vertex).or_insert_with(|| { 
									vertex_list.push(RawObjVertex {
										position: *position_data.get(vertex.0).ok_or(()).unwrap(),
										texture: vertex.1.map_or(None, |i| Some(*texture_data.get(i)?)),
										normal: vertex.2.map_or(None, |i| Some(*normal_data.get(i)?)),
									}); 
									vertex_list.len() - 1 
								})
							}
						};
					}

					group.push([
						extract_vertex!(vertices[0]),
						extract_vertex!(vertices[1]),
						extract_vertex!(vertices[2]),
					]);
					vertices.remove(1);
				}
			}
		}

		Ok(objects.into_iter().map(|(name, groups, _, vertex_list)| {
			let mut group_info = Box::new_uninit_slice(groups.len());
			let mut current_index = 0;
			for i in 0..groups.len() {
				let (group_name, indices) = &groups[i];
				let start = current_index * 3;
				let end = (current_index + indices.len()) * 3;

				current_index += indices.len();

				group_info[i].write((start, end, group_name.clone()));
			}
			let group_info = group_info.into_iter().map(|gi| unsafe { gi.assume_init() }).collect::<Box<_>>();

			let vertices = vertex_list.into_iter()
				.map(|v| v.into())
				.collect::<Box<[_]>>();

			let indices = groups.into_iter()
				.map(|(_, i)| i.into_flattened())
				.flatten()
				.map(|i| i as u32)
				.collect::<Box<[_]>>();

			ObjMesh {
				name: 		name,
				vertices: 	vertices,
				indices: 	indices,
				groups:		group_info,
			}
		}).collect::<Box<[_]>>())
	}
}
