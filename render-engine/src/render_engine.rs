
use std::{
	collections::{HashMap, HashSet}, 
	sync::{
        Arc, 
        mpsc::{Sender, SyncSender, TrySendError, channel, sync_channel},
    }, 
	thread::Builder as ThreadBuilder,
};

use sdl2::{
	EventPump, 
	Sdl, 
	VideoSubsystem, 
	event::Event,
};
use shaderc::Compiler;
use vulkano::instance::Instance;

use crate::{
	macros::error_map,
	render_engine::{
    	create_info::RenderThreadCreateInfo, 
		render_command::RenderEngineCommand, 
		render_thread::RenderThread,
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
	pub(crate) flags: RenderEngineFlags,
	pub(crate) event_senders: HashMap<u32, SyncSender<Event>>,

    pub(crate) spirv_compiler: Option<Compiler>,
	pub(crate) sdl_resources: Option<SdlResources>,

	pub(crate) instance: Arc<Instance>,
}

pub(crate) struct SdlResources {
	pub(crate) event_pump: EventPump,
	pub(crate) video: VideoSubsystem,
}

macro_rules! run_render_thread {
    ($create_info: ident, $command_channel: ident, $init_channel: ident) => {
        move || {
            let result = RenderThread::new($create_info, $command_channel);
            if result.is_err() {
                $init_channel.send(Err(unsafe { result.unwrap_err_unchecked() })).unwrap();
                return
			}
            let mut internal = result.unwrap();
            $init_channel.send(Ok(internal.instance.clone())).unwrap();

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
		let sdl_resources = SdlResources::from_sdl(&sdl);

		let render_thread_create_info = RenderThreadCreateInfo::new(app_name, &app_version, &flags, &sdl_resources);
        ThreadBuilder::new()
            .name("Render Thread".to_string())
            .spawn(run_render_thread!(render_thread_create_info, command_r, init_s))
            .map_err(error_map!())?;

        let instance = init_r.recv().unwrap()?;

        Ok(RenderEngine {
            command_channel: Arc::new(command_s),
			flags: flags,
			event_senders: HashMap::new(),

            spirv_compiler: spirv_compiler,
			sdl_resources: sdl_resources,

			instance: instance,
        })
    }

	pub fn poll_events(&mut self) {
		let event_pump = &mut self.sdl_resources.as_mut().unwrap().event_pump;
		let mut outdated_senders = HashSet::with_capacity(self.event_senders.len());

		for event in event_pump.poll_iter() {
			if let Some(id) = event.get_window_id() {
				let sender = self.event_senders.get(&id).unwrap();
				let result = sender.try_send(event);
				if let Err(TrySendError::Disconnected(_)) = result {
					outdated_senders.insert(*&id);
				}
			} else {
				for (id, sender) in &self.event_senders {
					let result = sender.try_send(event.clone());
					if let Err(TrySendError::Disconnected(_)) = result {
						outdated_senders.insert(*id);
					}
				}
			}
		}

		for id in outdated_senders {
			self.event_senders.remove(&id);
		}
	}
}

impl Drop for RenderEngine {
    fn drop(&mut self) {
        let _ = self.command_channel.send(
			RenderEngineCommand::Exit
		);
    }
}

impl SdlResources {
	fn from_sdl(sdl: &Option<Sdl>) -> Option<Self> {
		if sdl.is_none() { return None }

		let sdl = sdl.as_ref().unwrap();

		Some(SdlResources {
			event_pump: sdl.event_pump().unwrap(),
			video: sdl.video().unwrap(),
		})
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