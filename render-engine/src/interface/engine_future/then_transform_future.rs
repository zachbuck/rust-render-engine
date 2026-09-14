use crate::engine_future::EngineFuture;


pub struct ThenTransformFuture<F, P, T> 
where F: EngineFuture<P> {
	future:		F,
	transform: 	Option<Box<dyn FnOnce(P) -> T>>
}

impl<F, P, T> ThenTransformFuture<F, P, T>
where F: EngineFuture<P> {
	pub fn new<R>(previous: (F, R), transform: Box<dyn FnOnce(P) -> T>) -> (ThenTransformFuture<F, P, T>, R) {
		let (future, response) = previous;

		let future = ThenTransformFuture {
			future: future,
			transform: Some(transform),
		};

		(future, response)
	}
}

impl<F, P, T> EngineFuture<T> for ThenTransformFuture<F, P, T> 
where F: EngineFuture<P> {
	fn wait(self) -> T { (self.transform.unwrap())(self.future.wait()) }
	fn try_wait(&mut self) -> Option<T> { self.future.try_wait().map(|p| (self.transform.take().unwrap())(p)) }
}
