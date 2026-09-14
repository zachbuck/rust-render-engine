
pub struct RawObjVertex {
	position: 	[f32; 4],
	texture:	Option<[f32; 3]>,
	normal:		Option<[f32; 3]>,
}

pub struct ObjMesh<V> where
V: From<RawObjVertex> {
	vertices: 	Box<[V]>,
	indices: 	Box<[u32]>,
}

impl<V> ObjMesh<V> where 
V: From<RawObjVertex> {
	pub fn get_vertices(&self) -> &Box<[V]> { &self.vertices }
	pub fn get_indices(&self) -> &Box<[u32]> { &self.indices }

	pub fn parse(obj_file: &str) -> Self {
		todo!()
		/*
			Procedure
			- First pass
				- Vertex data parsed into memory
				- unique combinations of indices kept
				- array of faces with their vertex 
		 */
	}
}
