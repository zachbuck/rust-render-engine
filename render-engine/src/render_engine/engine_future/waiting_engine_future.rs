
use std::sync::{
	Arc, 
	mpsc::{Receiver, TryRecvError},
};

use vulkano::sync::{
	GpuFuture, 
	future::FenceSignalFuture,
};

use crate::render_engine::engine_future::EngineFuture;

pub struct WaitingEngineFuture<T> {
	condition: EngineWaitType,
	future: dyn EngineFuture<T>,
}

impl<T> EngineFuture<T> for WaitingEngineFuture<T> {
	fn wait(&mut self) -> T {
		todo!()
	}

	fn try_wait(&mut self) -> Result<T, ()> {
		todo!()
	}
}

impl<T> WaitingEngineFuture<T> {

}

enum EngineWaitType {
	GpuChannel(Receiver<Arc<FenceSignalFuture<Box<dyn GpuFuture + Send>>>>),
	GpuFuture(Arc<FenceSignalFuture<Box<dyn GpuFuture + Send>>>),
}

impl EngineWaitType {
	fn wait(wait: &mut EngineWaitType) { 
		match wait {
			EngineWaitType::GpuChannel(receiver) => {
				let fence = receiver.recv().unwrap();
				*wait = EngineWaitType::GpuFuture(fence);
				EngineWaitType::wait(wait);
			},
			EngineWaitType::GpuFuture(fence) => {
				fence.wait(None).unwrap();
			},
		}
	}

	fn is_complete(wait: &mut EngineWaitType) -> bool {
		match wait {
			EngineWaitType::GpuChannel(receiver) => {
				let result = receiver.try_recv();
				if result.as_ref().is_err_and(|e| *e == TryRecvError::Empty) { return false; }
				let fence = result.unwrap();
				*wait = EngineWaitType::GpuFuture(fence);
				EngineWaitType::is_complete(wait)
			},
			EngineWaitType::GpuFuture(fence) => {
				fence.is_signaled().unwrap()
			},
		}
	}
}

impl EngineWaitType {
	fn new_gpu_future(channel: Receiver<Arc<FenceSignalFuture<Box<dyn GpuFuture + Send>>>>) -> Self {
		EngineWaitType::GpuChannel(channel)
	}
}
