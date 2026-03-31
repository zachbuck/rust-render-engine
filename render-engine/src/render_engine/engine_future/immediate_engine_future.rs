
use crate::render_engine::engine_future::EngineFuture;

pub struct ImmediateEngineFuture<T> 
where T: Copy,
{
	data: T
}

impl<T> EngineFuture<T> for ImmediateEngineFuture<T> 
where T: Copy,
{
	fn wait(&mut self) -> T {
		return self.data
	}

	fn try_wait(&mut self) -> Result<T, ()> {
		return Ok(self.data)
	}
}

impl<T> ImmediateEngineFuture<T>
where T: Copy
{
	fn new(data: T) -> Self {
		ImmediateEngineFuture { data }
	}
}
