
use std::{
	sync::{
        Arc, 
        mpsc::{Sender, channel, sync_channel},
    }, 
	thread::Builder as ThreadBuilder,
};

use sdl2::Sdl;
use shaderc::Compiler;

use crate::{
	macros::error_map,
	render_engine::{
    	create_info::RenderThreadCreateInfo, render_command::RenderEngineCommand, render_thread::RenderThread
	}
};

pub use create_info::RenderEngineFlags as RenderEngineFlags;

pub mod engine_future;
pub(crate) mod render_command;
pub(crate) mod render_resources;
pub(crate) mod render_thread;
mod create_info;

pub struct RenderEngine {
    pub(crate) command_channel: Arc<Sender<RenderEngineCommand>>,

    pub(crate) spirv_compiler: Option<Compiler>,
	#[expect(dead_code)]
	pub(crate) sdl: Option<Sdl>,
}

macro_rules! run_render_thread {
    ($create_info: ident, $command_channel: ident, $init_channel: ident) => {
        move || {
            let result = RenderThread::new($create_info, $command_channel);
            if result.is_err() {
                $init_channel.send(Err(unsafe { result.unwrap_err_unchecked() })).unwrap();
                return
            }
            $init_channel.send(Ok(())).unwrap();

            let mut internal = result.unwrap();

            while !internal.should_close {
                internal.process_command();
            }
        }
    };
}

impl RenderEngine {
    pub fn new(app_name: &str, app_version: [u32; 3], flags: RenderEngineFlags) -> Result<Self, ()> {
        let (command_s, command_r) = channel();
        let (init_s, init_r) = sync_channel(1);

		let spirv_compiler = flags.generate_spirv_compiler();
		let sdl = flags.generate_sdl();

		let render_thread_create_info = RenderThreadCreateInfo::new(app_name, &app_version, &flags, &sdl);
        ThreadBuilder::new()
            .name("Render Thread".to_string())
            .spawn(run_render_thread!(render_thread_create_info, command_r, init_s))
            .map_err(error_map!())?;

        init_r.recv().unwrap()?;

        Ok(RenderEngine {
            command_channel: Arc::new(command_s),
            spirv_compiler: spirv_compiler,
			sdl: sdl,
        })
    }
}

impl Drop for RenderEngine {
    fn drop(&mut self) {
        self.command_channel.send(RenderEngineCommand::Exit).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use crate::render_engine::{RenderEngine, RenderEngineFlags};

	#[test]
	/// Ensure that `RenderEngine::new()` and `RenderEngine::drop()` are working as expected.
	fn new_render_engine() {
		let engine = RenderEngine::new("Render Engine", [0, 1, 0], RenderEngineFlags::empty()).unwrap();

		drop(engine)
	}
}