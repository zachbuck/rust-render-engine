
macro_rules! error_map {
	() => {
		|e| { 
			if cfg!(test) {
				println!("{:?}", e); 
			}
			() 
		}
	};
}

pub(crate) use error_map;