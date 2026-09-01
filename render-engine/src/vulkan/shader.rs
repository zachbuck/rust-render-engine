
use spir_v::shader::{SpirvShader, SpirvShaderInfo};
use uuid::Uuid;
use vulkano::shader::{EntryPoint, ShaderModuleCreateInfo};

use crate::{
	engine_command::ShaderCommand, 
	vulkan::render_thread::RenderThread,
};

pub struct ShaderModule {
	#[expect(unused)]
	entry_point: EntryPoint,
	#[expect(unused)]
	shader_info: SpirvShaderInfo,
}

impl RenderThread {
	pub fn process_shader_command(&mut self, command: Box<ShaderCommand>) -> () {
		match *command {
			ShaderCommand::CreateShader { source, response } => response.send(self.create_shader(source)),
			ShaderCommand::DropShader { uuid } => self.drop_shader(uuid),
		}
	}

	fn create_shader(&mut self, source: SpirvShader) -> Result<(Uuid,), ()> {
		let uuid = Uuid::now_v7();

		let module = unsafe { vulkano::shader::ShaderModule::new(
			self.device.clone(), 
			ShaderModuleCreateInfo::new(source.get_binary()),
		) }.map_err(|_| ())?;

		let entry_point = module.entry_point("main").ok_or(())?;

		let shader_info = source.discard_binary();

		let shader = ShaderModule {
			entry_point,
			shader_info,
		};

		self.shader_modules.insert(uuid, shader);

		return Ok((uuid,))
	}

	fn drop_shader(&mut self, uuid: Uuid) -> () {
		self.shader_modules.remove(&uuid);
	}
}
