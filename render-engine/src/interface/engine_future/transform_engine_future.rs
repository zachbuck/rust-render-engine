
use crate::interface::engine_future::EngineFuture;

pub struct TransformEngineFuture<F, P, T> 
where F: EngineFuture<P> {
	previous: F,
	transform: Option<Box<dyn FnOnce(P) -> T>>,
}

impl<F, P, T> TransformEngineFuture<F, P, T> 
where F: EngineFuture<P> {
	pub fn new<R>(previous: (F, R), transform: Box<dyn FnOnce(P) -> T>) -> (TransformEngineFuture<F, P, T>, R) {
		let (future, response) = previous;

		let future = TransformEngineFuture {
			previous: future,
			transform: Some(transform),
		};

		(future, response)
	}
}

impl<F, P, T> EngineFuture<T> for TransformEngineFuture<F, P, T> 
where F: EngineFuture<P> {
	fn unwrap(self) -> T { self.transform.unwrap()(self.previous.unwrap()) }
	fn try_unwrap(&mut self) -> Result<T, ()> { self.previous.try_unwrap().map(|p| self.transform.take().unwrap()(p)) }
}
