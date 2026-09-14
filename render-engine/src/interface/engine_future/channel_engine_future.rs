
use std::sync::mpsc::{Receiver, SyncSender, TryRecvError, sync_channel};

use crate::engine_future::EngineFuture;

pub struct ChannelEngineFuture<T> {
	channel: Receiver<T>,
}

#[derive(Debug)]
pub struct ChannelEngineResponse<T> {
	channel: SyncSender<T>,
}

impl<T> ChannelEngineFuture<T> {
	pub fn new() -> (ChannelEngineFuture<T>, ChannelEngineResponse<T>) {
		let (sender, receiver) = sync_channel(1);

		let future = ChannelEngineFuture {
			channel: receiver,
		};

		let response = ChannelEngineResponse {
			channel: sender,
		};

		(future, response)
	}
}

impl<T> ChannelEngineResponse<T> {
	pub fn send(self, data: T) -> () {
		let _ = self.channel.send(data);
	}
}

impl<T> EngineFuture<T> for ChannelEngineFuture<T> {
	fn wait(self) -> T { self.channel.recv().unwrap() }

	fn try_wait(&mut self) -> Option<T> {
		let result = self.channel.try_recv();
		if let Err(TryRecvError::Empty) = result { return None }
		return Some(result.unwrap());
	}
}
