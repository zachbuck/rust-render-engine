
use std::sync::{
	Arc, 
	mpsc::{Sender, sync_channel},
};

use shaderc::ShaderKind;
use uuid::Uuid;
use vulkano::shader::{
	ShaderStages, 
	spirv::ExecutionModel,
};

use crate::{
	macros::error_map, 
	render_engine::{
		RenderEngine, 
		engine_future::EngineFuture, 
		render_command::RenderEngineCommand,
	}, 
	shader::{
		descriptor_requirements::DescriptorRequirements, 
		shader_command::ShaderCommand,
	},
};

pub(crate) mod descriptor_requirements;
pub(crate) mod shader_internal;
pub(crate) mod shader_command;

#[derive(Debug)]
pub struct Shader {
	pub(crate) uuid: Uuid,
	command_channel: Arc<Sender<RenderEngineCommand>>,

	pub shader_type: ShaderType,
	pub(crate) descriptor_requirements: DescriptorRequirements,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
#[derive(Hash)]
#[derive(Clone, Copy)]
#[derive(Debug)]
#[repr(u8)]
pub enum ShaderType {
	Vertex		= 0x1,
	Fragment	= 0x2,
}

impl Shader { 
	pub fn compile(render_engine: &RenderEngine, shader_name: &str, shader_type: ShaderType, shader_source: &str) -> Result<Box<[u32]>, ()> {
		let compiler = render_engine.spirv_compiler.as_ref().ok_or(())?;

		let artifact = compiler.compile_into_spirv(
			shader_source, 
			shader_type.into(), 
			shader_name, 
			"main", 
			None
		).map_err(error_map!())?;

		return Ok(artifact.as_binary().to_owned().into_boxed_slice());
	}

	pub fn new(render_engine: &RenderEngine, binary: Box<[u32]>) -> EngineFuture<Result<Arc<Self>, ()>> {
		let (send, recv) = sync_channel(1);

		render_engine.command_channel.send(
			ShaderCommand::CreateShader { 
				sender: send, 
				binary, 
				command_channel: render_engine.command_channel.clone(), 
			}.into()
		).unwrap();

		return EngineFuture::new_single(recv);
	}

	pub fn get_all(render_engine: &RenderEngine) -> EngineFuture<Result<Box<[Arc<Shader>]>, ()>> {
		let (send, recv) = sync_channel(1);

		render_engine.command_channel.send(
			ShaderCommand::GetShaders { 
				sender: send
			}.into()
		).unwrap();
		
		EngineFuture::new_single(recv)
	}
}

impl Drop for Shader {
	fn drop(&mut self) {
		self.command_channel.send(
			ShaderCommand::DropShader { 
				uuid: self.uuid,
			}.into()
		).unwrap();
	}
}

impl Into<ShaderKind> for ShaderType {
	fn into(self) -> ShaderKind {
		match self {
			ShaderType::Vertex 		=> ShaderKind::Vertex,
			ShaderType::Fragment 	=> ShaderKind::Fragment,
		}
	}
}

impl From<ExecutionModel> for ShaderType {
	fn from(value: ExecutionModel) -> Self {
		match value {
			ExecutionModel::Vertex		=> ShaderType::Vertex,
			ExecutionModel::Fragment 	=> ShaderType::Fragment,
			_ => todo!("Currently only Vertex and Fragment shaders are currently supported."),
		}
	}
}

impl Into<ShaderStages> for ShaderType {
	fn into(self) -> ShaderStages {
		match self {
			ShaderType::Vertex 		=> ShaderStages::VERTEX,
			ShaderType::Fragment 	=> ShaderStages::FRAGMENT,
		}
	}
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
		render_engine::{RenderEngine, RenderEngineFlags}, 
		shader::{Shader, ShaderType},
	};

	const VERTEX_SOURCE: &str = "
		#version 460

		void main() {
			gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
		}
	";

	const VERTEX_BINARY: [u32; 188] = [
		0x07230203, 0x00010000, 0x000D000B, 0x00000015, 0x00000000, 
		0x00020011, 0x00000001, 0x0006000B, 0x00000001, 0x4C534C47, 
		0x6474732E, 0x3035342E, 0x00000000, 0x0003000E, 0x00000000, 
		0x00000001, 0x0006000F, 0x00000000, 0x00000004, 0x6E69616D, 
		0x00000000, 0x0000000D, 0x00030003, 0x00000002, 0x000001CC, 
		0x000A0004, 0x475F4C47, 0x4C474F4F, 0x70635F45, 0x74735F70, 
		0x5F656C79, 0x656E696C, 0x7269645F, 0x69746365, 0x00006576, 
		0x00080004, 0x475F4C47, 0x4C474F4F, 0x6E695F45, 0x64756C63, 
		0x69645F65, 0x74636572, 0x00657669, 0x00040005, 0x00000004, 
		0x6E69616D, 0x00000000, 0x00060005, 0x0000000B, 0x505F6C67, 
		0x65567265, 0x78657472, 0x00000000, 0x00060006, 0x0000000B, 
		0x00000000, 0x505F6C67, 0x7469736F, 0x006E6F69, 0x00070006, 
		0x0000000B, 0x00000001, 0x505F6C67, 0x746E696F, 0x657A6953, 
		0x00000000, 0x00070006, 0x0000000B, 0x00000002, 0x435F6C67, 
		0x4470696C, 0x61747369, 0x0065636E, 0x00070006, 0x0000000B, 
		0x00000003, 0x435F6C67, 0x446C6C75, 0x61747369, 0x0065636E, 
		0x00030005, 0x0000000D, 0x00000000, 0x00030047, 0x0000000B, 
		0x00000002, 0x00050048, 0x0000000B, 0x00000000, 0x0000000B, 
		0x00000000, 0x00050048, 0x0000000B, 0x00000001, 0x0000000B, 
		0x00000001, 0x00050048, 0x0000000B, 0x00000002, 0x0000000B,
		0x00000003, 0x00050048, 0x0000000B, 0x00000003, 0x0000000B, 
		0x00000004, 0x00020013, 0x00000002, 0x00030021, 0x00000003,
		0x00000002, 0x00030016, 0x00000006, 0x00000020, 0x00040017, 
		0x00000007, 0x00000006, 0x00000004, 0x00040015, 0x00000008,
		0x00000020, 0x00000000, 0x0004002B, 0x00000008, 0x00000009, 
		0x00000001, 0x0004001C, 0x0000000A, 0x00000006, 0x00000009,
		0x0006001E, 0x0000000B, 0x00000007, 0x00000006, 0x0000000A, 
		0x0000000A, 0x00040020, 0x0000000C, 0x00000003, 0x0000000B,
		0x0004003B, 0x0000000C, 0x0000000D, 0x00000003, 0x00040015, 
		0x0000000E, 0x00000020, 0x00000001, 0x0004002B, 0x0000000E,
		0x0000000F, 0x00000000, 0x0004002B, 0x00000006, 0x00000010, 
		0x00000000, 0x0004002B, 0x00000006, 0x00000011, 0x3F800000,
		0x0007002C, 0x00000007, 0x00000012, 0x00000010, 0x00000010, 
		0x00000010, 0x00000011, 0x00040020, 0x00000013, 0x00000003,
		0x00000007, 0x00050036, 0x00000002, 0x00000004, 0x00000000, 
		0x00000003, 0x000200F8, 0x00000005, 0x00050041, 0x00000013,
		0x00000014, 0x0000000D, 0x0000000F, 0x0003003E, 0x00000014, 
		0x00000012, 0x000100FD, 0x00010038, 
	];

	const ERROR_SOURCE: &str = "
		#version 460

		void main() {
			gl_Position = vec4(0.0, 0.0, 0.0);
		}
	";

	#[test]
	/// Ensure that `Shader::compile()` is working as expected.
	fn compile_shader() {
		let engine_flags = RenderEngineFlags {
			feature_spirv_compiler: true,
			..Default::default()
		};
		let engine = RenderEngine::new("Shader Test", [0, 1, 0], engine_flags).unwrap();

		let binary = Shader::compile(&engine, "vertex.glsl.vert", ShaderType::Vertex, VERTEX_SOURCE).unwrap();

		assert!(binary.len() == VERTEX_BINARY.len());
		for i in 0..binary.len() {
			assert!(binary[i] == VERTEX_BINARY[i]);
		}
	}

	#[test]
	/// Ensure that `Shader::compile()` returns `Err(())` on incorrect code submission.
	fn compile_shader_incorrect_code() {
		let engine_flags = RenderEngineFlags {
			feature_spirv_compiler: true,
			..Default::default()
		};
		let engine = RenderEngine::new("Shader Test", [0, 1, 0], engine_flags).unwrap();

		let result = Shader::compile(&engine, "vertex.glsl.vert", ShaderType::Vertex, ERROR_SOURCE);

		assert!(result.is_err_and(|e| e == ()));
	}

	#[test]
	/// Ensure that `Shader::compile()` returns `Err(())` if `RenderEngine` is created without a SPIR-V compiler.
	fn compile_shader_no_spirv_compiler() {
		let engine = RenderEngine::new("Shader Test", [0, 1, 0], RenderEngineFlags::empty()).unwrap();

		let result = Shader::compile(&engine, "vertex.glsl.vert", ShaderType::Vertex, VERTEX_SOURCE);

		assert!(result.is_err_and(|e| e == ()));
	}

	#[test]
	/// Ensure that `Shader::new()` and `Shader::drop()` are working as expected.
	fn new_shader() {
		let engine_flags = RenderEngineFlags {
			feature_spirv_compiler: true,
			..Default::default()
		};
		let engine = RenderEngine::new("Shader Test", [0, 1, 0], engine_flags).unwrap();

		let shader = Shader::new(&engine, Box::new(VERTEX_BINARY)).unwrap().unwrap();

		drop(shader);

		let shader_list = Shader::get_all(&engine).unwrap().unwrap();

		assert!(shader_list.len() == 0);
	}

	#[test]
	/// Ensure that `Shader::get_all()` is working as expected.
	fn get_all() {
		let engine_flags = RenderEngineFlags {
			feature_spirv_compiler: true,
			..Default::default()
		};
		let engine = RenderEngine::new("Shader Test", [0, 1, 0], engine_flags).unwrap();

		let shader = Shader::new(&engine, Box::new(VERTEX_BINARY)).unwrap().unwrap();

		let shader_list = Shader::get_all(&engine).unwrap().unwrap();

		assert!(shader_list.len() == 1);
		assert!(Arc::ptr_eq(&shader, &shader_list[0]));
	}
}