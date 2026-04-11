use std::{fs::File, io::Read};

use file_parsing::spirv::{SpirvParser, SpirvParserFlags};
use shaderc::{Compiler, ShaderKind};

const SHADER_PATH: &str = "./render-engine/tests/rotated_triangle/vertex.glsl.vert";

fn main() {
	let compiler = Compiler::new().unwrap();

	let mut file = File::open(SHADER_PATH).unwrap();
	let mut source = String::new();
	file.read_to_string(&mut source).unwrap();

	let binary = compiler.compile_into_spirv(&source, ShaderKind::Vertex, "AAA", "main", None).unwrap();

	let parsed = SpirvParser::parse(binary.as_binary(), SpirvParserFlags { descriptors: true });
	for descriptor in &parsed.get_descriptors().descriptors {
		println!("{:?}", descriptor);
	}
}