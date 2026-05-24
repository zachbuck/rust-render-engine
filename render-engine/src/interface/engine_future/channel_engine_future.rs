
use std::{
	sync::mpsc::{Receiver, RecvTimeoutError, SyncSender, sync_channel}, 
	time::Duration,
};

use crate::interface::engine_future::EngineFuture;

pub struct ChannelEngineFuture<T> {
	channel: Receiver<T>,
}

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

impl<T> EngineFuture<T> for ChannelEngineFuture<T> {
	fn unwrap(self) -> T { self.channel.recv().unwrap() }
	fn try_unwrap(&mut self) -> Result<T, ()> {
		let response = self.channel.recv_timeout(Duration::from_nanos(1));
		if let Err(RecvTimeoutError::Timeout) = response { return Err(()) }
		Ok(response.unwrap())
	}
}
