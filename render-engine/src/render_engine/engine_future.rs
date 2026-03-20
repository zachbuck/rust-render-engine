
use std::sync::mpsc::{Receiver, sync_channel};

#[derive(Debug)]
#[must_use]
pub struct EngineFuture<T> {
    channel: Receiver<T>,
}

impl<T> EngineFuture<T> {
    pub fn try_unwrap(self) -> Result<T, ()> { self.channel.try_recv().map_err(|_| ()) }
    pub fn unwrap(self) -> T { self.channel.recv().unwrap() }

    pub(crate) fn new(channel: Receiver<T>) -> Self { EngineFuture { channel } }
	pub(crate) fn new_immediate(value: T) -> Self { 
		let (send, recv) = sync_channel(1);
		send.send(value).unwrap();
		EngineFuture { channel: recv }
	}
}
