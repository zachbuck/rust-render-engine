
use crate::render_engine::engine_future::{EngineFuture, EngineFutureBuilder};

pub struct ImmediateEngineFuture<T> {
	data: T
}

impl<T> EngineFuture<T> for ImmediateEngineFuture<T> 
where T: Clone {
	fn wait(&mut self) -> T {
		return self.data.clone()
	}

	fn try_wait(&mut self) -> Result<T, ()> {
		return Ok(self.data.clone())
	}
}

impl<F: Clone + 'static> EngineFutureBuilder<F> {
	pub(crate) fn new_immediate(data: F) -> Self {
		let engine_future = Box::new(ImmediateEngineFuture { data: data });
		EngineFutureBuilder { engine_future: engine_future }
	}
}