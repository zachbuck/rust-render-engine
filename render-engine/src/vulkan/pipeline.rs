
use uuid::Uuid;

use crate::{
	interface::{
		engine_command::PipelineCommand,
	},
	vulkan::{
		render_engine::RenderEngine,
	},
};

pub struct Pipeline {
	surface: Uuid,
	shaders: Box<[Uuid]>,
}

impl RenderEngine {
	pub fn process_pipeline_command(&mut self, command: Box<PipelineCommand>) {
		match *command {
			PipelineCommand::CreatePipeline { surface, shaders, response } => response.send(self.create_pipeline(surface, shaders)),
			PipelineCommand::DropPipeline { uuid } => self.drop_pipeline(uuid),
		}
	}

	fn create_pipeline(&mut self, surface: Uuid, shaders: Box<[Uuid]>) -> Result<(Uuid,), ()> {
		println!("Pipeline::new");

		let uuid = Uuid::now_v7();

		let pipeline = Pipeline {
			surface,
			shaders,
		};
		self.pipelines.insert(uuid, pipeline);

		Ok((uuid,))
	}

	fn drop_pipeline(&mut self, uuid: Uuid) -> () {
		println!("Pipeline::drop");

		self.pipelines.remove(&uuid);
	}
}
