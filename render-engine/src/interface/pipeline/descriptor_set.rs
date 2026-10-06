
use std::sync::{
	Arc, 
	mpsc::Sender,
};

use parsing::spir_v::shader::DescriptorBinding;
use uuid::Uuid;

use crate::{
	engine_command::EngineCommand, 
	engine_future::{
		EngineFuture, 
		channel_engine_future::ChannelEngineFuture, 
		then_transform_future::ThenTransformFuture,
	}, 
	pipeline::Pipeline,
};
  pub struct DescriptorSet {
	uuid: Uuid,
	pub pipeline: Arc<Pipeline>,

	pub set: u32,
	pub bindings: Vec<DescriptorBinding>,
}

impl DescriptorSet {
	pub fn generate_descriptor_set(pipeline: &Arc<Pipeline>, set: u32) -> impl EngineFuture<Result<Arc<DescriptorSet>, ()>> {
		let pipeline_pointer = pipeline.clone();
		let (future, result) = ThenTransformFuture::new(
			ChannelEngineFuture::new(), 
			Box::new(|result: Result<_, _>| {
				result.map(|(uuid, bindings)| {
					DescriptorSet {
						uuid,
						pipeline: pipeline_pointer,
						set,
						bindings,
					}
				})
			})
		);

		let _ = pipeline.command_channel.send(todo!());

		todo!() as Box<dyn EngineFuture<_>>
	}
}

impl Drop for DescriptorSet {
	fn drop(&mut self) {
		let _ = self.pipeline.command_channel.send(todo!());
	}
}
