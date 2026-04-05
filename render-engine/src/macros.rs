
macro_rules! error_map {
	() => {
		|e| { 
			if cfg!(debug_assertions) {
				println!("ERROR at `{}:{}:{}:`: \n\t{:?}", file!(), line!(), column!(), e); 
			}
			() 
		}
	};
}

pub(crate) use error_map;

#[allow(unused)]
macro_rules! count_passings {
	($str: literal) => {
		#[allow(static_mut_refs)]
		unsafe {
			use std::time::{Duration, Instant};

			static mut START: Option<Instant> = None;
			static mut COUNT: u32 = 0;

			if START.is_none() { START = Some(Instant::now()); }

			COUNT += 1;

			if Instant::now() > START.unwrap() + Duration::from_secs(1) {
				println!("{}: {}", $str, COUNT);
				COUNT = 0;
				START = Some(Instant::now());
			}
		}
	};
}

#[allow(unused)]
pub(crate) use count_passings;