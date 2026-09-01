
use crate::{
	data_type::DataType, 
	enumerations::ExecutionModel, 
	interpreter::Interpreter,
};

#[derive(Debug)]
pub struct SpirvShader {
	binary: 			Box<[u32]>,
	shader_stage: 		ShaderStage,
	inputs:				Box<[DataType]>,
	outputs: 			Box<[DataType]>,
	uniforms: 			Box<[DescriptorSet]>,
}

pub struct SpirvShaderInfo {
	shader_stage: 		ShaderStage,
	inputs:				Box<[DataType]>,
	outputs:			Box<[DataType]>,
	uniforms:			Box<[DescriptorSet]>,
}

#[derive(Clone, Copy)]
#[derive(Debug)]
pub enum ShaderStage {
	Unknown,
	Vertex,
	Fragment,
}

#[derive(Debug)]
pub struct DescriptorSet {
	pub set: u32,
	pub bindings: Box<[DescriptorBinding]>
}

#[derive(Debug)]
pub struct DescriptorBinding {
	pub binding: u32,
	pub data_type: DataType,
}

impl SpirvShader {
	pub unsafe fn from_binary(binary: Box<[u32]>) -> Self {
		let stage = Interpreter::get_shader_stage(&binary);

		let (inputs, outputs, uniforms) = Interpreter::get_variable_layout(&binary);

		SpirvShader {
			binary: 		binary,
			shader_stage: 	stage,
			inputs:			inputs,
			outputs:		outputs,
			uniforms:		uniforms,
		}
	}

	pub fn discard_binary(self) -> SpirvShaderInfo {
		SpirvShaderInfo { 
			shader_stage: 	self.shader_stage,
			inputs: 		self.inputs,
			outputs: 		self.outputs,
			uniforms: 		self.uniforms,
		}
	}

	pub fn get_binary(&self) -> &[u32] { &self.binary }
	pub fn get_stage(&self) -> &ShaderStage { &self.shader_stage }
	pub fn get_inputs(&self) -> &[DataType] { &self.inputs }
	pub fn get_outputs(&self) -> &[DataType] { &self.outputs }
	pub fn get_uniforms(&self) -> &[DescriptorSet] { &self.uniforms }
}

impl SpirvShaderInfo {
	pub fn get_stage(&self) -> &ShaderStage { &self.shader_stage }
	pub fn get_inputs(&self) -> &[DataType] { &self.inputs }
	pub fn get_outputs(&self) -> &[DataType] { &self.outputs }
	pub fn get_uniforms(&self) -> &[DescriptorSet] { &self.uniforms }
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

impl From<ExecutionModel> for ShaderStage {
	fn from(value: ExecutionModel) -> Self {
		match value {
			ExecutionModel::Vertex 		=> ShaderStage::Vertex,
			ExecutionModel::Fragment 	=> ShaderStage::Fragment,
		}
	}
}
