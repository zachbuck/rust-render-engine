
use std::sync::Arc;

use sdl3::video::{Window, WindowBuilder};
use uuid::Uuid;
use vulkano::{
	command_buffer::{
		AutoCommandBufferBuilder, 
		CommandBufferUsage, 
		PrimaryAutoCommandBuffer, 
		RenderPassBeginInfo, 
		SubpassBeginInfo, 
		SubpassEndInfo, 
		allocator::StandardCommandBufferAllocator,
	}, 
	device::Queue, 
	format::{ClearValue, Format}, 
	image::{
		ImageUsage, 
		view::ImageView,
	}, 
	pipeline::graphics::viewport::Viewport, 
	render_pass::{Framebuffer, FramebufferCreateInfo}, 
	swapchain::{
		ColorSpace, 
		PresentMode, 
		Surface as VSurface, 
		Swapchain, 
		SwapchainAcquireFuture, 
		SwapchainCreateInfo, 
		SwapchainPresentInfo, 
		acquire_next_image,
	}, 
	sync::GpuFuture,
};

use crate::{
	engine_command::window_surface_command::WindowSurfaceCommand, 
	macros::{debug_error, debug_none}, 
	surface::{
		RenderPassCreateInfo, 
		window_surface::WindowSurfaceCreateInfo,
	}, 
	vulkan::{
		render_object::RenderObject, 
		render_thread::{Operation, OperationType, RenderResources, RenderThread}, 
		surface::Surface
	},
};

/* TODO 
	- [ ] Add viewports
*/
pub struct WindowSurface {
	#[expect(unused)]
	window:			Window,
	render_pass:	Uuid,
	#[expect(unused)]
	vulkan_surface:	Arc<VSurface>,
	swapchain: 		Arc<Swapchain>,
	framebuffers:	Box<[Arc<Framebuffer>]>,
	futures:		Box<[Operation]>,

	// Render Context for the current frame
	builder:			Option<AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>>,
	index: 				Option<u32>,
	recreate_swapchain: bool,
	acquire_future:		Option<SwapchainAcquireFuture>,

	clear_color:		[f32; 4],
}

impl Surface for WindowSurface {
	fn begin_rendering(&mut self, allocator: &Arc<StandardCommandBufferAllocator>, graphics_queue: &Arc<Queue>) -> Result<(), ()> {
		let mut builder = AutoCommandBufferBuilder::primary(
			allocator.clone(), 
			graphics_queue.queue_family_index(), 
			CommandBufferUsage::OneTimeSubmit,
		).map_err(debug_error!())?;

		if self.recreate_swapchain { todo!() /* Recreate swapchain */ }

		let (index, suboptimal, acquire_future) = acquire_next_image(self.swapchain.clone(), None).map_err(debug_error!())?;
		self.index = Some(index);
		self.recreate_swapchain = suboptimal;
		self.acquire_future = Some(acquire_future);

		let framebuffer = &self.framebuffers[index as usize];
		self.futures[index as usize].cleanup_finished();

		builder
			.begin_render_pass(
				RenderPassBeginInfo {
					clear_values: vec![
						Some(ClearValue::Float(self.clear_color)),
					], // TODO: Add Clear Values
					..RenderPassBeginInfo::framebuffer(framebuffer.clone())
				},
				SubpassBeginInfo::default()
			).map_err(debug_error!())?;

		builder.set_viewport(0, vec![Viewport {
			offset: [0.0, 0.0],
			extent: [framebuffer.extent()[0] as f32, framebuffer.extent()[1] as f32],
			depth_range: 0.0..=1.0
		}].into()).map_err(debug_error!())?;
		self.builder = Some(builder);

		Ok(())
	}

	fn end_rendering(&mut self, graphics_queue: &Arc<Queue>, previous_operation: Operation) -> Result<Operation, ()> {
		let mut builder = self.builder.take().unwrap();
		let index = self.index.unwrap() as usize;

		builder
			.end_render_pass(
				SubpassEndInfo::default()
			).map_err(debug_error!())?;

		let command_buffer = builder
			.build()
			.map_err(debug_error!())?;

		let frame_operation = &mut self.futures[index];

		let mut future = frame_operation.future.take().map(|f| f.boxed_send());

		// Signal semaphore if needed (i.e., operation is crossing queues)
		if future.is_some() && frame_operation.needs_semaphore(OperationType::Graphics) {
			future = Some(future.unwrap().then_signal_semaphore().boxed_send());
		}

		// Join frame's previous operation and queue's previous operation
		if future.is_some() && previous_operation.future.is_some() {
			future = Some(future.unwrap().join(previous_operation.future.unwrap()).boxed_send());
		} else if previous_operation.future.is_some() {
			future = Some(previous_operation.future.unwrap().clone().boxed_send());
		}

		// Join Swapchain Acquisition Future
		if future.is_some() {
			future = Some(future.unwrap().join(self.acquire_future.take().unwrap()).boxed_send());
		} else {
			future = Some(self.acquire_future.take().unwrap().boxed_send());
		}

		// At this point the future definitely exists so take it out of the Option
		let future = future.unwrap(); 

		let future = Arc::new(future
			.then_execute(graphics_queue.clone(), command_buffer).map_err(debug_error!())?
			.then_swapchain_present(graphics_queue.clone(), SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), self.index.unwrap())).boxed_send()
			.then_signal_fence_and_flush().map_err(debug_error!())?);

		*frame_operation = Operation::graphics(future);

		return Ok(frame_operation.clone());
	}

	fn render_object(&mut self, render_object: &RenderObject, render_resources: &mut RenderResources) -> Result<(), ()> {
		let mut builder = self.builder.take().unwrap();
		
		let mesh_data = render_resources.mesh_data.get_mut(&render_object.mesh_data).unwrap();
		mesh_data.bind(&mut builder)?;

		let pipeline = render_resources.pipelines.get(self.get_renderpass()).unwrap().get(&render_object.pipeline).unwrap();
		builder
			.bind_pipeline_graphics(pipeline.pipeline.clone()).map_err(debug_error!())?;

		unsafe { builder
			.draw_indexed(mesh_data.indices.len() as u32, 1, 0, 0, 0).map_err(debug_error!())?;
		}

		self.builder = Some(builder);

		Ok(())
	}

	fn get_renderpass(&self) -> &Uuid { &self.render_pass }
}

impl RenderThread {
	pub fn process_window_surface_command(&mut self, command: Box<WindowSurfaceCommand>) -> () {
		match *command {
			WindowSurfaceCommand::CreateWindowSurface { create_info, render_pass_info, response } => response.send(self.create_window_surface(create_info, render_pass_info)),
			WindowSurfaceCommand::DropWindowSurface { uuid } => self.drop_window_surface(uuid),
		}
	}

	fn create_window_surface(&mut self, create_info: WindowSurfaceCreateInfo, render_pass_info: RenderPassCreateInfo) -> Result<(Uuid,), ()> {
		let uuid = Uuid::now_v7();

		let window = WindowBuilder::new(&self.video, &create_info.title, create_info.dimensions[0], create_info.dimensions[1])
			.build().map_err(debug_error!())?;

		let vulkan_surface = unsafe { VSurface::from_window_ref(self.instance.clone(), &window).map_err(debug_error!())? };

		let surface_capabilities = self.device
			.physical_device()
			.surface_capabilities(&vulkan_surface, Default::default())
			.map_err(debug_error!())?;

		let surface_formats = self.device
			.physical_device()
			.surface_formats(&vulkan_surface, Default::default())
			.map_err(debug_error!())?;

		let (format, color_space) = surface_formats
			.iter()
			.find(|(f, cs)| *f == Format::R8G8B8A8_UNORM && *cs == ColorSpace::SrgbNonLinear)
			.ok_or_else(debug_none!())?;

		let render_pass_uuid = self.get_renderpass(render_pass_info)?;
		let render_pass = self.render_passes.get(&render_pass_uuid).unwrap();

		let (swapchain, images) = Swapchain::new(
			self.device.clone(), 
			vulkan_surface.clone(), 
			SwapchainCreateInfo {
				min_image_count: surface_capabilities.min_image_count.max(2),
				image_format: *format,
				image_color_space: *color_space,
				image_extent: [window.size().0, window.size().1],
				image_usage: ImageUsage::COLOR_ATTACHMENT,
				present_mode: PresentMode::Fifo,
				..Default::default()
			},
		).map_err(debug_error!())?;

		let mut framebuffers = Vec::with_capacity(images.len());
		for x in 0..images.len() {
			let output = ImageView::new_default(images[x].clone()).map_err(debug_error!())?;

			framebuffers.push(Framebuffer::new(
				render_pass.clone(), 
				FramebufferCreateInfo {
					attachments: vec![
						output
					],
					extent: [window.size().0, window.size().1],
					..Default::default()
				},
			).map_err(debug_error!())?);
		}
		let framebuffers = framebuffers.into_boxed_slice();

		let mut futures = Vec::with_capacity(images.len());
		for _ in 0..images.len() {
			futures.push(Operation { operation_type: OperationType::Graphics, future: None });
		}
		let futures = futures.into_boxed_slice();

		self.surfaces.insert(uuid, Box::new(WindowSurface {
			window:				window,
			render_pass:		render_pass_uuid,
			vulkan_surface:		vulkan_surface,
			swapchain: 			swapchain,
			framebuffers: 		framebuffers,
			futures: 			futures,

			builder: 			None,
			index: 				None,
			recreate_swapchain: false,
			acquire_future: 	None,

			clear_color:		create_info.clear_color,
		}));

		Ok((uuid,))
	}

	fn drop_window_surface(&mut self, uuid: Uuid) -> () {
		let _window_surface = self.surfaces.remove(&uuid).unwrap();
		// TODO: drop renderpass objects as needed, and unneeded pipelines.
	}
}
