
use std::{
	collections::{HashMap, HashSet}, 
	sync::{
		Arc, 
		mpsc::Receiver,
	}, 
	time::Duration,
};

use crate::{
	interface::{
		RenderEngineCreateInfo,
		engine_command::{EngineCommand, InstructionBufferCommand},
		instruction_buffer::{Instruction, InstructionBuffer}
	}, 
	vulkan::{
		mesh_data::MeshData, 
		pipeline::Pipeline, 
		render_target::RenderTarget, 
		shader::Shader, 
		surface::Surface,
	}
};

pub struct RenderEngine {
	command_channel: Receiver<EngineCommand>,
	exit_received: bool,

	pub mesh_data: HashMap<Uuid, MeshData>,
	pub pipelines: HashMap<Uuid, Pipeline>,
	pub render_targets: HashMap<Uuid, Box<dyn RenderTarget>>,
	pub shaders: HashMap<Uuid, Shader>,
	pub surfaces: HashMap<Uuid, Box<dyn Surface>>,

	graphics_queue_family_index: u32,

	command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
}

macro_rules! render_thread {
	($create_info: ident, $command_channel: ident, $engine_response: ident) => {
		{
			use crate::vulkan::render_engine::RenderEngine as VRenderEngine;

			let engine = VRenderEngine::new($create_info, $command_channel);
			if let Err(e) = engine { $engine_response.send(Err(e)); }
			let mut engine = engine.unwrap();
			$engine_response.send(Ok(()));
			
			while !engine.should_close() {
				engine.process_event();
			}

			println!("RenderEngine::drop")
		}
	};
}
pub(crate) use render_thread;
use uuid::Uuid;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, allocator::StandardCommandBufferAllocator};

impl RenderEngine {
	pub fn new(create_info: RenderEngineCreateInfo, command_channel: Receiver<EngineCommand>) -> Result<Self, ()> {
		println!("RenderEngine::new");

		Ok(RenderEngine {
			command_channel: 			command_channel,
			exit_received: 				false,

			mesh_data:					HashMap::new(),
			pipelines:					HashMap::new(),
			render_targets:				HashMap::new(),
			shaders:					HashMap::new(),
			surfaces:					HashMap::new(),

			graphics_queue_family_index: todo!(),

			command_buffer_allocator: 	todo!(),
		})
	}

	pub fn process_event(&mut self) {
		match self.command_channel.recv_timeout(Duration::from_nanos(1)) {
			Ok(EngineCommand::DropEngine) => self.exit_received = true,
			Ok(EngineCommand::ImageSurfaceCommand(command)) => self.process_image_surface_command(command),
			Ok(EngineCommand::InstructionBufferCommand(command)) => self.process_instruction_buffer(command),
			Ok(EngineCommand::MeshDataCommand(command)) => self.process_mesh_data_command(command),
			Ok(EngineCommand::PipelineCommand(command)) => self.process_pipeline_command(command),
			Ok(EngineCommand::RenderObjectCommand(command)) => self.process_render_object_command(command),
			Ok(EngineCommand::ShaderCommand(command)) => self.process_shader_command(command),
			Err(_) => (),
		}
	}

	fn process_instruction_buffer(&mut self, command: Box<InstructionBufferCommand>) {
		match *command {
			InstructionBufferCommand::OneTimeSubmit { instructions, response } => response.send(self.one_time_submit(instructions)),
		}
	}

	fn one_time_submit(&mut self, instructions: InstructionBuffer) -> Result<(), ()> {
		let mut builder = AutoCommandBufferBuilder::primary(
			self.command_buffer_allocator.clone(), 
			self.graphics_queue_family_index, 
			CommandBufferUsage::OneTimeSubmit,
		).unwrap();

		let mut affected_resources = HashSet::new();

		for instruction in &instructions.instructions {
			match instruction {
				Instruction::BeginSurface(uuid) => {
					affected_resources.insert(*uuid);
					let surface = self.surfaces.get_mut(uuid).unwrap();
					surface.begin_rendering(&mut builder)?;
				},
				Instruction::DrawRenderTarget(uuid) => {
					println!("InstructionBuffer::DrawRenderTarget");
				},
				Instruction::EndSurface(uuid) => {
					affected_resources.insert(*uuid);
					let surface = self.surfaces.get_mut(uuid).unwrap();
					surface.begin_rendering(&mut builder)?;
				},
			}
		}

		return Ok(())
	}

	pub fn should_close(&mut self) -> bool { self.exit_received }
}
