
use std::sync::mpsc::TryRecvError;

use crate::{
    mesh_data::mesh_data_command::MeshDataCommand, 
    pipeline::pipeline_command::PipelineCommand, 
    render_engine::render_thread::RenderThread, 
    render_surface::{
        image_surface::image_surface_commands::ImageSurfaceCommand, 
        render_surface_command::RenderSurfaceCommand, 
		window_surface::window_surface_command::WindowSurfaceCommand
    }, 
    renderable::render_object::render_object_command::RenderObjectCommand, 
    shader::shader_command::ShaderCommand, 
    texture::texture_command::TextureCommand
};

#[derive(Debug)]
pub(crate) enum RenderEngineCommand {
    Exit,

    MeshDataCommand(MeshDataCommand),
    ShaderCommand(ShaderCommand),
	PipelineCommand(PipelineCommand),
    TextureCommand(TextureCommand),

    RenderObjectCommand(RenderObjectCommand),

    RenderSurfaceCommand(RenderSurfaceCommand),
    ImageSurfaceCommand(ImageSurfaceCommand),
	WindowSurfaceCommand(WindowSurfaceCommand),
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
            RenderEngineCommand::TextureCommand(command) => self.process_texture_command(command),

            RenderEngineCommand::RenderObjectCommand(command) => self.process_render_object_command(command),

            RenderEngineCommand::RenderSurfaceCommand(command) => self.process_render_surface_command(command),
            RenderEngineCommand::ImageSurfaceCommand(command) => self.process_image_surface_command(command),
			RenderEngineCommand::WindowSurfaceCommand(command) => self.process_window_surface_command(command),
        }
    }

    fn process_exit(&mut self) {
        self.should_close = true;
    }
}
