
macro_rules! debug_error {
	() => {
		{
			#[cfg(not(debug_assertions))]
			{ |_| { () } }

			#[cfg(debug_assertions)]
			{ |e| {
				println!("{:?}", e);
				println!("\tat {}:{}:{}:", file!(), line!(), column!());
				()
			} }
		}
	};
}
pub(crate) use debug_error;

macro_rules! debug_none {
	() => {
		{
			#[cfg(not(debug_assertions))]
			{ || { () } }

			#[cfg(debug_assertions)]
			{ || {
				println!("None Variant unwrapped");
				println!("\tat {}:{}:{}:", file!(), line!(), column!());
				()
			} }
		}
	};
}
pub(crate) use debug_none;
