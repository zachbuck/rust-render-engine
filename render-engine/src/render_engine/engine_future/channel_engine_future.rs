
use std::sync::mpsc::Receiver;

use crate::render_engine::engine_future::{EngineFuture, EngineFutureBuilder};

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

impl<F: 'static> EngineFutureBuilder<F> {
	pub(crate) fn new_channel(recv: Receiver<F>) -> Self {
		let engine_future = Box::new(ChannelEngineFuture { channel: recv });
		EngineFutureBuilder { engine_future: engine_future }
	}
}
