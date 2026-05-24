
use std::{
	marker::PhantomData, 
	sync::mpsc::{Receiver, RecvTimeoutError, SyncSender, sync_channel}, 
	time::Duration,
};

use crate::interface::engine_future::EngineFuture;

pub struct WaitEngineFuture<F, T>
where F: EngineFuture<T> {
	out_type: PhantomData<T>,
	future: F,
	condition: WaitConditionState,
}

enum WaitConditionState {
	Immediate(Box<dyn WaitCondition>),
	Channel(Receiver<Box<dyn WaitCondition>>),
}

pub trait WaitCondition {
	fn wait(&mut self) -> ();
	fn is_finished(&mut self) -> bool;
}

pub struct ImmediateWait;

pub struct WaitFutureResponse<R> {
	sender: SyncSender<Box<dyn WaitCondition>>,
	response: R,
}

impl<F, T> WaitEngineFuture<F, T>
where F: EngineFuture<T> {
	pub fn immediate<R>(previous: (F, R), condition: Box<dyn WaitCondition>) -> (WaitEngineFuture<F, T>, R) {
		let (future, response) = previous;

		let future = WaitEngineFuture {
    		out_type: PhantomData,
			future,
			condition: WaitConditionState::Immediate(condition),
		};

		(future, response)
	}

	pub fn channel<R>(previous: (F, R)) -> (WaitEngineFuture<F, T>, WaitFutureResponse<R>) {
		let (future, response) = previous;

		let (sender, receiver) = sync_channel(1);

		let future = WaitEngineFuture {
			out_type: PhantomData,
			future,
			condition: WaitConditionState::Channel(receiver)
		};

		let response = WaitFutureResponse {
			sender,
			response
		};

		(future, response)
	}
}

impl<F, T> EngineFuture<T> for WaitEngineFuture<F, T> 
where F: EngineFuture<T> {
	fn unwrap(mut self) -> T { self.condition.wait(); self.future.unwrap() }
	fn try_unwrap(&mut self) -> Result<T, ()> {
		if self.condition.is_finished() {
			self.future.try_unwrap()
		} else {
			Err(())
		}
	}
}

impl WaitConditionState {
	pub fn wait(&mut self) {
		match self {
			WaitConditionState::Immediate(wait_condition) => wait_condition.wait(),
			WaitConditionState::Channel(receiver) => {
				let wait_condition = receiver.recv().unwrap();
				*self = WaitConditionState::Immediate(wait_condition);
				self.wait();
			},
		}
	}

	pub fn is_finished(&mut self) -> bool {
		match self {
			WaitConditionState::Immediate(wait_condition) => wait_condition.is_finished(),
			WaitConditionState::Channel(receiver) => {
				let response = receiver.recv_timeout(Duration::from_nanos(1));
				if let Err(RecvTimeoutError::Timeout) = response { return false }
				let wait_condition = response.unwrap();
				*self = WaitConditionState::Immediate(wait_condition);
				self.is_finished()
			},
		}
	}
}

impl WaitCondition for ImmediateWait {
	fn wait(&mut self) -> () { () }
	fn is_finished(&mut self) -> bool { true }
}

impl<R> WaitFutureResponse<R> {
	pub fn send(self, condition: Box<dyn WaitCondition>) -> R {
		let _ = self.sender.send(condition);
		self.response
	}
}
