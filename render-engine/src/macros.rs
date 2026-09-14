
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
