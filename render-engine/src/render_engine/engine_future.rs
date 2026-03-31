
mod channel_engine_future;
mod immediate_engine_future;
mod then_transform_future;
mod waiting_engine_future;

pub trait EngineFuture<T> {
	fn wait(&mut self) -> T;
	fn try_wait(&mut self) -> Result<T, ()>;
}

impl<T> EngineFuture<T> for Box<dyn EngineFuture<T>> {
	fn wait(&mut self) -> T {
		(self.as_mut()).wait()
	}

	fn try_wait(&mut self) -> Result<T, ()> {
		(self.as_mut()).try_wait()
	}
}

pub(crate) struct EngineFutureBuilder<F> {
	engine_future: Box<dyn EngineFuture<F>>,
}

impl<T> EngineFutureBuilder<T> {
	pub(crate) fn build(self) -> impl EngineFuture<T> {
		self.engine_future
	}
}
