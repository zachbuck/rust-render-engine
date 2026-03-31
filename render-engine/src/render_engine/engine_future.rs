
mod channel_engine_future;
mod immediate_engine_future;
mod then_transform_future;
mod waiting_engine_future;

pub(crate) use channel_engine_future::ChannelEngineFuture as ChannelEngineFuture;
pub(crate) use immediate_engine_future::ImmediateEngineFuture as ImmediateEngineFuture;
pub(crate) use waiting_engine_future::WaitingEngineFuture as WaitingEngineFuture;

pub trait EngineFuture<T> {
	fn wait(&mut self) -> T;
	fn try_wait(&mut self) -> Result<T, ()>;
}
