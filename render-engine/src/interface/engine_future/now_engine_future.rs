
use crate::engine_future::EngineFuture;

pub struct NowEngineFuture<T> {
	data: Option<T>
}

impl<T> NowEngineFuture<T> {
	#[expect(unused)]
	pub fn new(data: T) -> Self {
		NowEngineFuture { data: Some(data) }
	}
}

impl<T> EngineFuture<T> for NowEngineFuture<T> {
	fn wait(self) -> T {
		return self.data.unwrap()
	}
	
	fn try_wait(&mut self) -> Option<T> {
		return Some(self.data.take().unwrap())
	}
}
