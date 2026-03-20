
use std::sync::mpsc::Receiver;

pub struct EngineFuture<T> {
	future_type: EngineFutureType<T>,
}

enum EngineFutureType<T> {
	Immediate(T),
	Single(Receiver<T>),
}

impl<T> EngineFuture<T> {
	pub fn unwrap(self) -> T {
		match self.future_type {
			EngineFutureType::Immediate(data) => data,
			EngineFutureType::Single(channel) => channel.recv().unwrap(),
		}
	}

	pub fn try_unwrap(self) -> Result<T, ()> {
		match self.future_type {
			EngineFutureType::Immediate(data) => Ok(data),
			EngineFutureType::Single(channel) => Ok(channel.try_recv().map_err(|_| ())?),
		}
	}

	pub(crate) fn new_immediate(data: T) -> Self {
		EngineFuture { 
			future_type: EngineFutureType::Immediate(data)
		}
	}

	pub(crate) fn new_single(channel: Receiver<T>) -> Self {
		EngineFuture { 
			future_type: EngineFutureType::Single(channel) 
		}
	}
}
