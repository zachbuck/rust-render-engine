
use std::sync::mpsc::TryRecvError;

use crate::{
    mesh_data::mesh_data_command::MeshDataCommand, 
    pipeline::pipeline_command::PipelineCommand, 
    render_engine::render_thread::RenderThread, 
    shader::shader_command::ShaderCommand
};

#[derive(Debug)]
pub(crate) enum RenderEngineCommand {
    Exit,
    MeshDataCommand(MeshDataCommand),
    ShaderCommand(ShaderCommand),
	PipelineCommand(PipelineCommand),
}

impl RenderThread {
    pub(super) fn process_command(&mut self) {
        let result = self.command_channel.try_recv();
        let command;
        match result {
            Ok(rec) => command = rec,
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Disconnected) => {self.process_exit(); return},
        }

        match command {
            RenderEngineCommand::Exit => self.process_exit(),
            RenderEngineCommand::MeshDataCommand(command) => self.process_mesh_data_command(command),
            RenderEngineCommand::ShaderCommand(command) => self.process_shader_command(command),
			RenderEngineCommand::PipelineCommand(command) => self.process_pipeline_command(command),
        }
    }

    fn process_exit(&mut self) {
        self.should_close = true;
    }
}
