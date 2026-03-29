
use std::{
	fmt::Debug, sync::{
        Arc, 
        mpsc::{Sender, channel, sync_channel},
    }, thread::Builder as ThreadBuilder
};

use shaderc::Compiler;

use crate::{
	macros::error_map,
	render_engine::{
		create_info::RenderEngineCreateInfoFlags, 
    	render_command::RenderEngineCommand, 
    	render_thread::RenderThread,
	}
};

pub use create_info::RenderEngineCreateInfo as RenderEngineCreateInfo;

pub mod engine_future;
pub(crate) mod render_command;
pub(crate) mod render_resources;
pub(crate) mod render_thread;
mod create_info;

#[derive(Debug)]
pub struct RenderEngine {
    pub(crate) command_channel: Arc<Sender<RenderEngineCommand>>,
    pub(crate) spirv_compiler: Option<Compiler>,
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
    pub fn new(create_info: RenderEngineCreateInfo) -> Result<Self, ()> {
        let (command_s, command_r) = channel();
        let (init_s, init_r) = sync_channel(1);

        let flags = create_info.flags;

        ThreadBuilder::new()
            .name("Render Thread".to_string())
            .spawn(run_render_thread!(create_info, command_r, init_s))
            .map_err(error_map!())?;

        init_r.recv().unwrap()?;

        let compiler;
        if flags & RenderEngineCreateInfoFlags::InitSpirvCompiler as u64 != 0 {
            compiler = Some(Compiler::new().map_err(error_map!())?)
        } else {
            compiler = None;
        }

        Ok(RenderEngine {
            command_channel: Arc::new(command_s),
            spirv_compiler: compiler,
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
    use crate::render_engine::{RenderEngine, RenderEngineCreateInfo};

	#[test]
	/// Ensure that `RenderEngine::new()` and `RenderEngine::drop()` are working as expected.
	fn new_render_engine() {
		let create_info = RenderEngineCreateInfo::new()
			.with_app_name("Test".to_string())
			.with_app_vers(1, 0, 0)
			.with_spirv_compiler();
		let engine = RenderEngine::new(create_info).unwrap();

		drop(engine)
	}
}