
use std::sync::{
	Arc, 
	mpsc::{Receiver, TryRecvError},
};

use vulkano::sync::{
	GpuFuture, 
	future::FenceSignalFuture,
};

use crate::render_engine::engine_future::{EngineFuture, EngineFutureBuilder};

pub struct WaitingEngineFuture<T> {
	condition: EngineWaitType,
	future: Box<dyn EngineFuture<T>>,
}

impl<T> EngineFuture<T> for WaitingEngineFuture<T> {
	fn wait(&mut self) -> T {
		EngineWaitType::wait(&mut self.condition);
		self.future.wait()
	}

	fn try_wait(&mut self) -> Result<T, ()> {
		let done = EngineWaitType::is_complete(&mut self.condition);
		if !done { return Err(()) }
		self.future.try_wait()
	}
}

enum EngineWaitType {
	GpuChannel(Receiver<Option<Arc<FenceSignalFuture<Box<dyn GpuFuture + Send>>>>>),
	GpuFuture(Arc<FenceSignalFuture<Box<dyn GpuFuture + Send>>>),
	Complete,
}

impl EngineWaitType {
	fn wait(wait: &mut EngineWaitType) { 
		match wait {
			EngineWaitType::GpuChannel(receiver) => {
				let fence = receiver.recv().unwrap();
				if fence.is_none() { 
					*wait = EngineWaitType::Complete;
					EngineWaitType::wait(wait);
				} else {
					let fence = fence.unwrap();
					*wait = EngineWaitType::GpuFuture(fence);
					EngineWaitType::wait(wait);
				}
			},
			EngineWaitType::GpuFuture(fence) => {
				fence.wait(None).unwrap();
			},
			EngineWaitType::Complete => { return }
		}
	}

	fn is_complete(wait: &mut EngineWaitType) -> bool {
		match wait {
			EngineWaitType::GpuChannel(receiver) => {
				let result = receiver.try_recv();
				if result.as_ref().is_err_and(|e| *e == TryRecvError::Empty) { return false; }
				let fence = result.unwrap();
				if fence.is_none() { 
					*wait = EngineWaitType::Complete;
					EngineWaitType::is_complete(wait)
				} else {
					let fence = fence.unwrap();
					*wait = EngineWaitType::GpuFuture(fence);
					EngineWaitType::is_complete(wait)
				}
			},
			EngineWaitType::GpuFuture(fence) => {
				fence.is_signaled().unwrap()
			},
			EngineWaitType::Complete => { true }
		}
	}
}

impl<T: 'static> EngineFutureBuilder<T> {
	pub(crate) fn with_gpu_future(self, recv: Receiver<Option<Arc<FenceSignalFuture<Box<dyn GpuFuture + Send>>>>>) -> Self {
		let engine_future = Box::new(WaitingEngineFuture { condition: EngineWaitType::GpuChannel(recv), future: self.engine_future });
		EngineFutureBuilder { engine_future: engine_future }
	}
}
