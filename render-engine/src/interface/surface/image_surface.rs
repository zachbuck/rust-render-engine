
use std::{
	marker::PhantomData, 
	sync::Arc,
};

use crate::{interface::
	engine_future::{
		EngineFuture,
		immediate_engine_future::ImmediateEngineFuture,
	}, 
	surface::{Surface, SurfaceInfo},
};

pub struct ImageSurface<P> {
	pixel_format: PhantomData<P>,
}

impl<P> ImageSurface<P> {
	pub fn new() -> impl EngineFuture<Result<Arc<ImageSurface<P>>, ()>> {
		todo!() as ImmediateEngineFuture<_>
	}
}

impl<P> Drop for ImageSurface<P> {
	fn drop(&mut self) {
		todo!()
	}
}

impl<P> Surface for ImageSurface<P> {
	
}

impl<P> SurfaceInfo for ImageSurface<P> {

}
