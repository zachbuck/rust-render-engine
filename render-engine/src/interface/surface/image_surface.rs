
use std::{
	marker::PhantomData, 
	sync::{
		Arc, 
		mpsc::Sender,
	},
};

use uuid::Uuid;

use crate::interface::{
	RenderEngine,
	data_format::pixel::PixelFormat,
	engine_command::{EngineCommand, ImageSurfaceCommand},
	engine_future::{
		EngineFuture,
		channel_engine_future::ChannelEngineFuture,
		transform_engine_future::TransformEngineFuture,
	}, 
	surface::{Surface, SurfaceInfo},
};

pub struct ImageSurface<P> 
where P: PixelFormat {
	pixel_format: PhantomData<P>,
	
	uuid: Uuid,
	command_channel: Sender<EngineCommand>,
}

impl<P> ImageSurface<P> 
where P: PixelFormat {
	pub fn new(render_engine: &RenderEngine, dimensions: [u32; 2]) -> impl EngineFuture<Result<Arc<ImageSurface<P>>, ()>> {
		let command_channel = render_engine.command_channel.clone();

		let (future, response) = TransformEngineFuture::new(
			ChannelEngineFuture::<Result<_, _>>::new(), 
			Box::new(|result: _| -> _ {
				result.map(|(uuid,)| Arc::new(ImageSurface { pixel_format: PhantomData, uuid, command_channel }))
			}),
		);

		let command = ImageSurfaceCommand::CreateImageSurface {
			dimensions, 
			vulkan_format: P::VULKAN_FORMAT,
			response: response,
		};
		let command = EngineCommand::ImageSurfaceCommand(Box::new(command));
		let _ = render_engine.command_channel.send(command);

		return future;
	}
}

impl<P> Drop for ImageSurface<P> 
where P: PixelFormat {
	fn drop(&mut self) {
		let command = ImageSurfaceCommand::DropImageSurface { uuid: self.uuid };
		let command = EngineCommand::ImageSurfaceCommand(Box::new(command));
		let _ = self.command_channel.send(command);
	}
}

impl<P> Surface for ImageSurface<P> 
where P: PixelFormat {}

impl<P> SurfaceInfo for ImageSurface<P> 
where P: PixelFormat {
	fn get_command_channel(&self) -> &Sender<EngineCommand> { &self.command_channel }
	fn get_uuid(&self) -> &Uuid { &self.uuid }
}
