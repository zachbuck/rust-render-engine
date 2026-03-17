use std::{
    sync::{
        Arc, 
        mpsc::{Receiver, Sender, channel},
    },
    thread::Builder as ThreadBuilder
};

use vulkano::{
    device::{Device, Queue}, 
    sync::GpuFuture
};

pub struct RenderEngine {
    command_channel: Sender<()>,
    response_channel: Receiver<()>,
}

macro_rules! run_render_thread {
    ($command_channel: ident, $response_channel: ident, $init_channel: ident) => {
        move || {
            let result = RenderThread::new($command_channel, $response_channel);
            if result.is_err() {
                $init_channel.send(Err(unsafe { result.unwrap_err_unchecked() })).unwrap();
                return
            }
            $init_channel.send(Ok(())).unwrap();

            let _internal = result.unwrap();
        }
    };
}

impl RenderEngine {
    pub fn new() -> Result<Arc<Self>, ()> {
        let (command_s, command_r) = channel();
        let (response_s, response_r) = channel();
        let (init_s, init_r) = channel();

        ThreadBuilder::new()
            .name("Render Thread".to_string())
            .spawn(run_render_thread!(command_r, response_s, init_s))
            .map_err(|_| ())?;

        init_r.recv().unwrap()?;

        Ok(Arc::new(RenderEngine {
            command_channel: command_s,
            response_channel: response_r,
        }))
    }
}

struct RenderThread {
    command_channel: Receiver<()>,
    response_channel: Sender<()>,

    device: Arc<Device>,
    render_queue: Arc<Queue>,
    render_future: Option<Box<dyn GpuFuture>>, 
    transfer_queue: Arc<Queue>,
    transfer_future: Option<Box<dyn GpuFuture>>,
}

impl RenderThread {
    fn new(command_channel: Receiver<()>, response_channel: Sender<()>) -> Result<Self, ()> {
        todo!()
    }
}