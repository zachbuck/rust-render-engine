
use uuid::Uuid;

use crate::{
	interface::engine_command::ShaderCommand,
	vulkan::render_engine::RenderEngine,
};

pub struct Shader {

}

impl RenderEngine {
	pub fn process_shader_command(&mut self, command: Box<ShaderCommand>) {
		match *command {
			ShaderCommand::CreateShaderSpirv { binary, response } => response.send(self.create_shader_spirv(binary)),
			ShaderCommand::DropShader { uuid } => self.drop_shader(uuid),
		}
	}

	fn create_shader_spirv(&mut self, binary: Box<[u32]>) -> Result<(Uuid,), ()> {
		println!("Shader::new");

		let uuid = Uuid::now_v7();

		let shader = Shader {};
		self.shaders.insert(uuid, shader);

		Ok((uuid,))
	}

	fn drop_shader(&mut self, uuid: Uuid) -> () {
		println!("Shader::drop");

		self.shaders.remove(&uuid);
	}
}
