
macro_rules! error_to_unit_type {
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
pub(crate) use error_to_unit_type;

macro_rules! none_to_unit_type {
	() => {
		{
			#[cfg(not(debug_assertions))]
			{ () }

			#[cfg(debug_assertions)]
			{
				println!("None Variant unwrapped");
				println!("\tat {}:{}:{}:", file!(), line!(), column!());
				()
			}
		}
	};
}
pub(crate) use none_to_unit_type;
