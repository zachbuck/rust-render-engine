
mod compile_shader {
	use std::{
		fs::File, 
		io::Read
	};

	use crate::{
		RenderEngine, 
		RenderEngineCreateInfo, 
		shader::ShaderType
	};

	#[test]
	fn compile_shader() {
		let renderer = RenderEngine::new(RenderEngineCreateInfo::default()).unwrap();
		
		let shader_path = "tests/shader/compile_test.vert";
		let mut file = File::open(shader_path).unwrap();
		let mut shader_source = String::new();
		file.read_to_string(&mut shader_source).unwrap();

		let (binary, warnings) = renderer.compile_shader(shader_source, shader_path.to_string(), ShaderType::Vertex).unwrap();

		assert!(warnings.is_none());

		let mut file = File::open("tests/shader/compile_test.vert.spv").unwrap();
		let mut test_binary = Vec::new();
		file.read_to_end(&mut test_binary).unwrap();

		assert!(binary.len() * 4 == test_binary.len());
		for x in 0..binary.len() {
			let bytes = binary[x].to_le_bytes();

			assert!(bytes[0] == test_binary[4*x+0]);
			assert!(bytes[1] == test_binary[4*x+1]);
			assert!(bytes[2] == test_binary[4*x+2]);
			assert!(bytes[3] == test_binary[4*x+3]);
		}
	}

	#[test]
	fn incorrect_shader_source() {
		let renderer = RenderEngine::new(RenderEngineCreateInfo::default()).unwrap();

		let shader_path = "tests/shader/incorrect_compile_test.vert";
		let mut file = File::open(shader_path).unwrap();
		let mut shader_source = String::new();
		file.read_to_string(&mut shader_source).unwrap();
		let result = renderer.compile_shader(shader_source, shader_path.to_string(), ShaderType::Vertex);
		
		assert!(result.is_err());
	}
}

mod create_shader {
    use std::{
		fs::File, 
		io::Read
	};

    use crate::{
		RenderEngine, 
		RenderEngineCreateInfo, 
		shader::ShaderType
	};

	#[test]
	fn create_shader() {
		let mut renderer = RenderEngine::new(RenderEngineCreateInfo::default()).unwrap();

		let shader_path = "tests/shader/compile_test.vert";
		let mut file = File::open(shader_path).unwrap();
		let mut shader_source = String::new();
		file.read_to_string(&mut shader_source).unwrap();

		let (shader_binary, _) = renderer.compile_shader(shader_source, shader_path.to_string(), ShaderType::Vertex).unwrap();
		let _shader = renderer.create_shader(shader_binary).unwrap();
	}
}
