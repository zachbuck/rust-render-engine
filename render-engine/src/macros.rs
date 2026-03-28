
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