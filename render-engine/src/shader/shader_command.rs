
use std::sync::{
	Arc, 
	mpsc::SyncSender,
};

use uuid::Uuid;
use vulkano::shader::{ShaderModule, ShaderModuleCreateInfo};

use crate::{
	render_engine::{
		RenderEngine, 
		render_command::RenderEngineCommand, 
		render_thread::RenderThread
	}, 
	shader::{
		Shader, descriptor_requirements::DescriptorRequirements, shader_internal::ShaderInternal
	}
};

#[derive(Debug)]
pub(crate) enum ShaderCommand {
	CreateShader {
		sender: SyncSender<Result<Arc<Shader>, ()>>,

		binary: Box<[u32]>,
		engine: Arc<RenderEngine>,
	},
	DropShader {
		uuid: Uuid,
	},
}

impl Into<RenderEngineCommand> for ShaderCommand {
	fn into(self) -> RenderEngineCommand {
		RenderEngineCommand::ShaderCommand(self)
	}
}

impl RenderThread {
	pub(crate) fn process_shader_command(&mut self, command: ShaderCommand) {
		match command {
			ShaderCommand::CreateShader { sender, binary , engine} => { let _ = sender.send(self.create_shader(binary.as_ref(), engine)); },
			ShaderCommand::DropShader { uuid } => self.drop_shader(uuid),
		}
	}

	fn create_shader(&mut self, shader_binary: &[u32], engine: Arc<RenderEngine>) -> Result<Arc<Shader>, ()> {
		let uuid = Uuid::now_v7();
		
		let module = unsafe {
			ShaderModule::new(
				self.device.clone(),
				ShaderModuleCreateInfo::new(shader_binary)
			).map_err(|_| ())?
		};

		let entry_point = module.entry_point("main").unwrap();

		let internal = ShaderInternal { 
			entry_point: entry_point.clone(),
			descriptor_requirements: DescriptorRequirements::from_vulkano(&entry_point)?,
		};

		let shader_type = internal.get_shader_type();

		self.shaders.insert(uuid, internal);

		Ok(Arc::new(Shader { 
			uuid, 
			render_engine: engine, 
			shader_type: shader_type,
			descriptor_requirements: DescriptorRequirements::from_vulkano(&entry_point).unwrap(),
		}))
	}

	fn drop_shader(&mut self, uuid: Uuid) {
		self.shaders.remove(&uuid);
	}
}

