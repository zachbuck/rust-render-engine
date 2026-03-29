
use std::sync::{
	Arc, 
	mpsc::{Sender, SyncSender},
};

use uuid::Uuid;
use vulkano::shader::{ShaderModule, ShaderModuleCreateInfo};

use crate::{
	macros::error_map, 
	render_engine::{
		render_command::RenderEngineCommand, 
		render_thread::RenderThread,
	}, 
	shader::{
		Shader, 
		descriptor_requirements::DescriptorRequirements, 
		shader_internal::ShaderInternal,
	}
};

#[derive(Debug)]
pub(crate) enum ShaderCommand {
	CreateShader {
		sender: SyncSender<Result<Arc<Shader>, ()>>,

		binary: Box<[u32]>,
		command_channel: Arc<Sender<RenderEngineCommand>>,
	},
	GetShaders {
		sender: SyncSender<Result<Box<[Arc<Shader>]>, ()>>,
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
			ShaderCommand::CreateShader { sender, binary , command_channel} => { let _ = sender.send(self.create_shader(binary.as_ref(), command_channel)); },
			ShaderCommand::GetShaders { sender } => { let _ = sender.send(self.get_shaders()); }
			ShaderCommand::DropShader { uuid } => self.drop_shader(uuid),
		}
	}

	fn create_shader(&mut self, shader_binary: &[u32], command_channel: Arc<Sender<RenderEngineCommand>>) -> Result<Arc<Shader>, ()> {
		let uuid = Uuid::now_v7();

		let module = unsafe {
			ShaderModule::new(
				self.device.clone(),
				ShaderModuleCreateInfo::new(shader_binary)
			).map_err(error_map!())?
		};

		let entry_point = module.entry_point("main").ok_or(())?;
		
		let descriptor_requirements = DescriptorRequirements::from_binary(shader_binary, entry_point.info().execution_model.into());

		let shader_type = entry_point.info().execution_model.into();

		let reference = Arc::new(Shader {
			uuid,
			command_channel,
			shader_type: shader_type,
			descriptor_requirements: descriptor_requirements.clone(),
		});

		let internal = ShaderInternal { 
			reference: Arc::downgrade(&reference),

			entry_point: entry_point.clone(),
		};

		self.shaders.insert(uuid, internal);

		return Ok(reference)

	}

	fn get_shaders(&mut self) -> Result<Box<[Arc<Shader>]>, ()> {
		Ok(self.shaders.values().filter_map(|s| s.reference.upgrade()).collect())
	}

	fn drop_shader(&mut self, uuid: Uuid) {
		self.shaders.remove(&uuid);
	}
}

