
pub(crate) mod immediate_engine_future;

pub trait EngineFuture<T> {
	fn unwrap(self) -> T;
	fn try_unwrap(&mut self) -> Result<T, ()>;
}


