
pub(crate) mod channel_engine_future;
pub(crate) mod function_engine_future;
pub(crate) mod immediate_engine_future;
pub(crate) mod transform_engine_future;
pub(crate) mod wait_engine_future;

#[must_use]
pub trait EngineFuture<T> {
	fn unwrap(self) -> T;
	fn try_unwrap(&mut self) -> Result<T, ()>;
}

