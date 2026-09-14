
use std::fmt::Debug;

pub mod compiler;
pub mod data_type;
mod enumerations;
pub mod interpreter;
pub mod shader;

pub struct WarningResult<T, W, E> {
	pub result: Result<T, E>,
	pub warnings: Vec<W>,
}

impl<T, W, E> WarningResult<T, W, E> {
	pub(crate) fn new(result: Result<T, E>, warnings: Vec<W>) -> Self {
		WarningResult {
			result,
			warnings,
		}
	}
}

impl<T, W, E> WarningResult<T, W, E> 
where E: Debug {
	pub fn unwrap(self) -> T {
		self.result.unwrap()
	}
}
