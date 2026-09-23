
pub(crate) mod channel_engine_future;
pub(crate) mod now_engine_future;
pub(crate) mod then_transform_future;

pub trait EngineFuture<T> {
	fn wait(self) -> T;
	fn try_wait(&mut self) -> Option<T>;
}

/* TODO
	- [ ] I feel like there's a better way to be able to return `dyn EngineFuture<_>` from a function
*/
impl<T> EngineFuture<T> for Box<dyn EngineFuture<T>> {
	fn wait(mut self) -> T {
		loop {
			let result = self.try_wait();
			if result.is_some() {
				return result.unwrap();
			}
		}
	}
	fn try_wait(&mut self) -> Option<T> { 
		self.as_mut().try_wait() 
	}
}
