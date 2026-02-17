
// this file exists entirely to avoid copy pasting this macro over and over and over and over and over again while not publically exposing it

#[macro_export]
macro_rules! unwrap_result_or_none {
	($x:expr) => {
		{
			let value = $x;

			if value.is_err() { return Err(()); }

			value.unwrap()
		}
	};
}

#[macro_export]
macro_rules! unwrap_option_or_none {
	($x:expr) => {
		{
			let value = $x;

			if value.is_none() { return Err(()); }

			value.unwrap()
		}
	};
}