
use shaderc::{
	CompileOptions, 
	Compiler as SpirvCompiler, 
	EnvVersion, 
	SourceLanguage, 
	TargetEnv,
};

use crate::spirv::WarningResult;

pub struct Compiler {
	compiler: SpirvCompiler,
}

pub struct SpirvShader {
	pub binary: Box<[u32]>,
	pub shader_stage: ShaderStage
}

#[derive(Clone, Copy)]
pub enum ShaderStage {
	Unknown,
	Vertex,
	Fragment,
}

impl Compiler {
	pub fn new() -> Result<Self, ()> {
		let compiler = SpirvCompiler::new().map_err(|_| ())?;

		Ok(Compiler {
			compiler: compiler,
		})
	}

	/// TODO
	/// - Allow setting target Vulkan Version
	/// - Allow setting source language
	pub fn compile_from_source(&self, shader_name: &str, shader_type: ShaderStage, source: &str) -> WarningResult<SpirvShader, String, ()> {
		let result = CompileOptions::new().map_err(|_| ());
		if result.is_err() { return WarningResult::new(Err(unsafe { result.unwrap_err_unchecked() }), Vec::new()) }
		let mut options = result.unwrap();

		options.set_target_env(TargetEnv::Vulkan, EnvVersion::Vulkan1_3 as u32);
		options.set_source_language(SourceLanguage::GLSL);

		let result = self.compiler.compile_into_spirv(
			source, 
			shader_type.into(), 
			shader_name, 
			"main", 
			Some(&options),
		).map_err(|_| ());
		if result.is_err() { return WarningResult::new(Err(unsafe { result.unwrap_err_unchecked() }), Vec::new()) }
		let artifact = result.unwrap();

		let shader = SpirvShader {
			binary: 		artifact.as_binary().to_owned().into_boxed_slice(),
			shader_stage: 	shader_type,
		};

		let warnings;
		if artifact.get_num_warnings() == 0 {
			warnings = Vec::new();
		} else {
			warnings = artifact.get_warning_messages().split("\n").map(|s| s.to_string()).collect()
		}

		WarningResult::new(Ok(shader), warnings)
	}
}

impl SpirvShader {
	pub unsafe fn from_binary(binary: Box<[u32]>, stage: ShaderStage) -> Self {
		SpirvShader {
			binary: 		binary,
			shader_stage: 	stage,
		}
	}
}

impl Into<shaderc::ShaderKind> for ShaderStage {
	fn into(self) -> shaderc::ShaderKind {
		match self {
			ShaderStage::Unknown	=> shaderc::ShaderKind::DefaultCompute,
			ShaderStage::Vertex 	=> shaderc::ShaderKind::Vertex,
			ShaderStage::Fragment 	=> shaderc::ShaderKind::Fragment,
		}
	}
}
