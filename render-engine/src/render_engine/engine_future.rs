
use std::{
	fmt::{Debug, Formatter}, 
	sync::{
		Arc, 
		mpsc::Receiver
	}
};

use vulkano::sync::{
	GpuFuture, 
	future::FenceSignalFuture
};

#[derive(Debug)]
#[must_use]
pub struct EngineFuture<T> {
	future_type: EngineFutureType<T>,
}

enum EngineFutureType<T> {
	Immediate(T),
	Single(Receiver<T>),
	Function(Receiver<Box<dyn FnOnce() -> T + Send>>),

	Composite(EngineWaitType, Box<EngineFuture<T>>), 
}

pub(crate) enum EngineWaitType {
	GpuFuture(Receiver<Arc<FenceSignalFuture<Box<dyn GpuFuture + Send>>>>),
}

impl<T> EngineFuture<T> {
	pub fn unwrap(self) -> T {
		match self.future_type {
			EngineFutureType::Immediate(data) => data,
			EngineFutureType::Single(channel) => channel.recv().unwrap(),
			EngineFutureType::Function(channel) => channel.recv().unwrap()(),

			EngineFutureType::Composite(a, b) => { a.wait(); b.unwrap() }
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

	pub(crate) fn new_function(channel: Receiver<Box<dyn FnOnce() -> T + Send>>) -> Self {
		EngineFuture { future_type: EngineFutureType::Function(channel) }
	}

	pub(crate) fn with_wait_condition(self, condition: EngineWaitType) -> Self {
		EngineFuture { future_type: EngineFutureType::Composite(condition, Box::new(self)) }
	}
}

impl EngineWaitType {
	fn wait(self) -> () {
		match self {
			EngineWaitType::GpuFuture(future) => {
				let result = future.recv();
				if result.is_err() { return; }
				result.unwrap().wait(None).unwrap();
			},
		}
	}
}

impl From<Receiver<Arc<FenceSignalFuture<Box<dyn GpuFuture + Send>>>>> for EngineWaitType {
	fn from(value: Receiver<Arc<FenceSignalFuture<Box<dyn GpuFuture + Send>>>>) -> Self {
		EngineWaitType::GpuFuture(value)
	}
}

impl<T> Debug for EngineFutureType<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Immediate(_) => write!(f, "Immediate"),
			Self::Single(_) => write!(f, "Single"),
			Self::Function(_) => write!(f, "Function"),
			Self::Composite(a, b) => write!(f, "Composite<{:?}, {:?}>", a, b.future_type),
		}
	}
}

impl Debug for EngineWaitType {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::GpuFuture(_) => write!(f, "GpuFuture"),
		}
	}
}
