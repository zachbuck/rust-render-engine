
macro_rules! debug_error {
	() => {
		debug_error!(())
	};

	($error: expr) => {
		{
			#[cfg(not(debug_assertions))]
			{ |_| { $error } }

			#[cfg(debug_assertions)]
			{ |e| {
				println!("{:?}", e);
				println!("\tat {}:{}:{}:", file!(), line!(), column!());
				$error
			} }
		}
	}
}
pub(crate) use debug_error;

macro_rules! debug_none {
	() => {
		debug_none!(())
	};

	($error: expr) => {
		{
			#[cfg(not(debug_assertions))]
			{ || { $error } }

			#[cfg(debug_assertions)]
			{ || {
				println!("None Variant unwrapped");
				println!("\tat {}:{}:{}:", file!(), line!(), column!());
				$error
			} }
		}
	}
}
pub(crate) use debug_none;
