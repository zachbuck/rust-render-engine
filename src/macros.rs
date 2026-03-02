
#[macro_export]
macro_rules! unwrap_result_or_none {
	($x:expr) => {
		{
			let value = $x;

			if value.is_err() { return Err(()); }

			value.unwrap()
		}
	};

	($x:expr, $y:expr) => {
		{
			let value = $x;

			if value.is_err() { return Err($y); }

			value.unwrap()
		}
	}
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