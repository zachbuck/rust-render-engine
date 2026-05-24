
use crate::interface::engine_future::EngineFuture;

pub struct ImmediateEngineFuture<T> {
	pub data: Option<T>,
}

impl<T> ImmediateEngineFuture<T> {
	pub fn new(data: T) -> (Self, ()) {
		let future = ImmediateEngineFuture {
			data: Some(data),
		};

		let builder = ();

		(future, builder)
	}
}

impl<T> EngineFuture<T> for ImmediateEngineFuture<T> {
	fn unwrap(self) -> T { self.data.unwrap() }
	fn try_unwrap(&mut self) -> Result<T, ()> { Ok(self.data.take().unwrap()) }
}
