
use crate::render_engine::engine_future::{EngineFuture, EngineFutureBuilder};

pub struct ThenTransformFuture<F, T> {
	transform: Option<Box<dyn FnOnce(F) -> T>>,
	future: Box<dyn EngineFuture<F>>,
}

impl<F, T> EngineFuture<T> for ThenTransformFuture<F, T> {
	fn wait(&mut self) -> T {
		(self.transform.take().unwrap())(self.future.wait())
	}

	fn try_wait(&mut self) -> Result<T, ()> {
		let input = self.future.try_wait()?;
		Ok((self.transform.take().unwrap())(input))
	}
}

impl<F: 'static> EngineFutureBuilder<F> {
	pub(crate) fn then_transform<T: 'static>(self, transform: Box<dyn FnOnce(F) -> T>) -> EngineFutureBuilder<T> {
		let engine_future = Box::new(ThenTransformFuture { transform: Some(transform), future: self.engine_future });
		EngineFutureBuilder { engine_future: engine_future }
	}
}
