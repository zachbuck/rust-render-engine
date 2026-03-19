
use std::sync::{
	Arc, 
	mpsc::{SyncSender, sync_channel}
};

use shaderc::ShaderKind;
use uuid::Uuid;
use vulkano::shader::{EntryPoint, ShaderModule, ShaderModuleCreateInfo, spirv::ExecutionModel};

use crate::render_engine::{EngineFuture, RenderEngine, RenderEngineCommand, RenderThread};

#[derive(Debug)]
pub struct Shader {
	uuid: Uuid,
	render_engine: Arc<RenderEngine>,

	pub shader_type: ShaderType,
}

impl Shader { 
	pub fn compile(render_engine: Arc<RenderEngine>, shader_name: &str, shader_type: ShaderType, shader_source: &str) -> Result<Box<[u32]>, ()> {
		let compiler = render_engine.spirv_compiler.as_ref().ok_or(())?;

		let artifact = compiler.compile_into_spirv(
			shader_source, 
			shader_type.into(), 
			shader_name, 
			"main", 
			None
		).map_err(|_| ())?;

		return Ok(artifact.as_binary().to_owned().into_boxed_slice());
	}

	pub fn new(render_engine: Arc<RenderEngine>, binary: Box<[u32]>) -> EngineFuture<Result<Arc<Self>, ()>> {
		let (send, recv) = sync_channel(1);

		render_engine.command_channel.send(
			RenderEngineCommand::ShaderCommand(
				ShaderCommand::CreateShader { 
					sender: send, 
					binary, 
					engine: render_engine.clone(), 
				}
			)
		).unwrap();

		return EngineFuture::new(recv);
	}
}

impl Drop for Shader {
	fn drop(&mut self) {
		self.render_engine.command_channel.send(
			RenderEngineCommand::ShaderCommand(
				ShaderCommand::DropShader { 
					uuid: self.uuid,
				}
			)
		).unwrap();
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
#[derive(Hash)]
#[derive(Clone, Copy)]
#[derive(Debug)]
pub enum ShaderType {
	Vertex,
	Fragment,
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
			_ => panic!("Unknown ExecutionModel type: '{:?}'", value),
		}
	}
}

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

#[derive(Debug)]
pub(crate) struct ShaderInternal {
	pub(crate) entry_point: EntryPoint,
}

impl ShaderInternal {
	pub fn get_shader_type(&self) -> ShaderType { self.entry_point.info().execution_model.into() }
}

impl RenderThread {
	pub(crate) fn process_shader_command(&mut self, command: ShaderCommand) {
		match command {
			ShaderCommand::CreateShader { sender, binary , engine} => sender.send(self.create_shader(binary.as_ref(), engine)).unwrap(),
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

		let internal = ShaderInternal { entry_point };

		let shader_type = internal.get_shader_type();

		self.shaders.insert(uuid, internal);

		Ok(Arc::new(Shader { uuid, render_engine: engine, shader_type: shader_type }))
	}

	fn drop_shader(&mut self, uuid: Uuid) {
		self.shaders.remove(&uuid);
	}
}

impl RenderThread {
	#[inline]
	pub(crate) fn get_shader_internal(&self, reference: Arc<Shader>) -> Option<&ShaderInternal> { self.shaders.get(&reference.uuid) }
	#[inline]
	pub(crate) fn get_mut_shader_internal(&mut self, reference: Arc<Shader>) -> Option<&mut ShaderInternal> { self.shaders.get_mut(&reference.uuid) }
}
