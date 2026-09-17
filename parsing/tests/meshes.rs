
use std::{fs::File, io::Read};

use parsing::meshes::obj::{ObjMesh, RawObjVertex};

#[test]
fn main() {
	let mut file = File::open("tests/test.obj").unwrap();
	let mut contents = String::new();

	let _ = file.read_to_string(&mut contents).unwrap();

	let result:Box<[ObjMesh<RawObjVertex>]> = ObjMesh::parse(&contents).unwrap();

	println!("{:#?}", result);
}
