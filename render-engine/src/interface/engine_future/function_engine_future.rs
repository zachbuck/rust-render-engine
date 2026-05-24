
use std::{
	sync::mpsc::{Receiver, RecvTimeoutError, SyncSender, sync_channel}, 
	time::Duration,
};

use crate::engine_future::EngineFuture;

pub struct FunctionEngineFuture<T> {
	channel: Receiver<Box<dyn FnOnce() -> T>>,
}

pub struct FunctionEngineResponse<T> {
	channel: SyncSender<Box<dyn FnOnce() -> T>>,
}

impl<T> FunctionEngineFuture<T> {
	pub fn new() -> (FunctionEngineFuture<T>, FunctionEngineResponse<T>) {
		let (sender, receiver) = sync_channel(1);

		let future = FunctionEngineFuture {
			channel: receiver,
		};

		let response = FunctionEngineResponse {
			channel: sender,
		};

		(future, response)
	}
}

impl<T> EngineFuture<T> for FunctionEngineFuture<T> {
	fn unwrap(self) -> T { self.channel.recv().unwrap()() }
	fn try_unwrap(&mut self) -> Result<T, ()> {
		let response = self.channel.recv_timeout(Duration::from_nanos(1));
		if let Err(RecvTimeoutError::Timeout) = response { return Err(()) }
		Ok(response.unwrap()())
	}
}

impl<T> FunctionEngineResponse<T> {
	pub fn send(self, function: Box<dyn FnOnce() -> T>) {
		let _ = self.channel.send(function);
	}
}
