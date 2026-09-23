
use std::sync::{
	Arc, 
	mpsc::Sender,
};

use parsing::spir_v::{
	data_type::DataType, 
	shader::{DescriptorCollection, ShaderStage, SpirvShader, SpirvShaderInfo},
};
use uuid::Uuid;

use crate::{
	engine_command::{
		EngineCommand, 
		shader_command::ShaderCommand,
	}, 
	engine_future::{
		EngineFuture, 
		channel_engine_future::ChannelEngineFuture, 
		then_transform_future::ThenTransformFuture
	}, 
	render_engine::RenderEngine, 
};

/* TODO
	- [ ] Add support for more than Vertex and Fragment shaders
 */
#[derive(Debug)]
pub struct Shader {
	uuid: 				Uuid,
	command_channel: 	Sender<EngineCommand>,
	shader_info:		SpirvShaderInfo,
}

impl Shader {
	pub fn new(render_engine: &Arc<RenderEngine>, shader: SpirvShader) -> impl EngineFuture<Result<Arc<Shader>, ()>> {
		let command_channel = render_engine.command_channel.clone();
		let shader_info = shader.get_info();
		let (future, response) = ThenTransformFuture::new(
			ChannelEngineFuture::new(), 
			Box::new(move |result: Result<_, _>| result.map(
				|(uuid,)| Arc::new(Shader {
					uuid: 				uuid,
					command_channel: 	command_channel,
					shader_info:		shader_info,
				})
			))
		);

		let _ = render_engine.command_channel.send(ShaderCommand::CreateShader { 
			source: shader, 
			response,
		}.into());

		return future;
	}

	pub(crate) fn get_uuid(&self) -> &Uuid { &self.uuid }

	pub fn get_stage(&self) -> &ShaderStage { self.shader_info.get_stage() }
	pub(crate) fn get_inputs(&self) -> &[DataType] { self.shader_info.get_inputs() }
	pub(crate) fn get_outputs(&self) -> &[DataType] { self.shader_info.get_outputs() }
	pub(crate) fn get_uniforms(&self) -> &DescriptorCollection { self.shader_info.get_uniforms() }
}

impl Drop for Shader {
	fn drop(&mut self) {
		let _ = self.command_channel.send(ShaderCommand::DropShader {
			uuid: self.uuid,
		}.into());
	}
}
