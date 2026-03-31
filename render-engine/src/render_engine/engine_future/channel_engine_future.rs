
use std::sync::mpsc::Receiver;

use crate::render_engine::engine_future::EngineFuture;

pub struct ChannelEngineFuture<T> {
	channel: Receiver<T>,
}

impl<T> EngineFuture<T> for ChannelEngineFuture<T> {
	fn wait(&mut self) -> T {
		self.channel.recv().unwrap()
	}

	fn try_wait(&mut self) -> Result<T, ()> {
		self.channel.try_recv().map_err(|_| ())
	}
}

impl<T> ChannelEngineFuture<T> {
	fn new(channel: Receiver<T>) -> Self {
		ChannelEngineFuture { channel }
	}
}
