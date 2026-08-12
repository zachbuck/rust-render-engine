
pub(crate) mod channel_engine_future;
pub(crate) mod now_engine_future;
pub(crate) mod then_transform_future;

pub trait EngineFuture<T> {
	fn wait(self) -> T;
	fn try_wait(&mut self) -> Option<T>;
}
