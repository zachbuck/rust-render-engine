
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
		ImageLayout, 
		ImageUsage, 
		view::ImageView,
	}, 
	render_pass::{AttachmentDescription, AttachmentLoadOp, AttachmentReference, AttachmentStoreOp, Framebuffer, FramebufferCreateInfo, RenderPass, RenderPassCreateInfo, SubpassDescription}, 
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
	engine_command::WindowSurfaceCommand, 
	surface::window_surface::WindowSurfaceCreateInfo, 
	vulkan::{
		render_thread::{Operation, OperationType, RenderThread}, 
		surface::Surface
	},
};

// TODO: Add viewports
pub struct WindowSurface {
	#[expect(unused)]
	window:			Window,
	render_pass:	Arc<RenderPass>,
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
		).map_err(|_| ())?;

		if self.recreate_swapchain { todo!() /* Recreate swapchain */ }

		let (index, suboptimal, acquire_future) = acquire_next_image(self.swapchain.clone(), None).map_err(|_| ())?;
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
			).map_err(|e| println!("{e}"))?;

		self.builder = Some(builder);

		Ok(())
	}

	fn end_rendering(&mut self, graphics_queue: &Arc<Queue>, previous_operation: Operation) -> Result<Operation, ()> {
		let mut builder = self.builder.take().unwrap();
		let index = self.index.unwrap() as usize;

		builder
			.end_render_pass(
				SubpassEndInfo::default()
			).map_err(|_| ())?;

		let command_buffer = builder
			.build()
			.map_err(|_| ())?;

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
			.then_execute(graphics_queue.clone(), command_buffer).map_err(|_| ())?
			.then_swapchain_present(graphics_queue.clone(), SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), self.index.unwrap())).boxed_send()
			.then_signal_fence_and_flush().map_err(|_| ())?);

		*frame_operation = Operation::graphics(future);

		return Ok(frame_operation.clone());
	}

	fn get_renderpass(&self) -> &Arc<RenderPass> { &self.render_pass }
}

impl RenderThread {
	pub fn process_window_surface_command(&mut self, command: Box<WindowSurfaceCommand>) -> () {
		match *command {
			WindowSurfaceCommand::CreateWindowSurface { create_info, render_pass_info: _, response } => response.send(self.create_window_surface(create_info)),
			WindowSurfaceCommand::DropWindowSurface { uuid } => self.drop_window_surface(uuid),
		}
	}

	fn create_window_surface(&mut self, create_info: WindowSurfaceCreateInfo) -> Result<(Uuid,), ()> {
		let uuid = Uuid::now_v7();

		let window = WindowBuilder::new(&self.video, &create_info.title, create_info.dimensions[0], create_info.dimensions[1])
			.build().map_err(|_| ())?;

		let vulkan_surface = unsafe { VSurface::from_window_ref(self.instance.clone(), &window).map_err(|_| ())? };

		let surface_capabilities = self.device
			.physical_device()
			.surface_capabilities(&vulkan_surface, Default::default())
			.map_err(|_| ())?;

		let surface_formats = self.device
			.physical_device()
			.surface_formats(&vulkan_surface, Default::default())
			.map_err(|_| ())?;

		let (format, color_space) = surface_formats
			.iter()
			.find(|(f, cs)| *f == Format::R8G8B8A8_UNORM && *cs == ColorSpace::SrgbNonLinear)
			.ok_or(())?;

		let render_pass = RenderPass::new(
			self.device.clone(), 
			RenderPassCreateInfo {
				attachments: vec![
					AttachmentDescription {
						format: 			Format::R8G8B8A8_UNORM,
						load_op:			AttachmentLoadOp::Clear,
						store_op: 			AttachmentStoreOp::Store,
						final_layout:		ImageLayout::ColorAttachmentOptimal,
						..Default::default()
					},
				],
				subpasses: vec![
					SubpassDescription {
						input_attachments: Vec::new(),
						color_attachments: vec![
							Some(AttachmentReference {
								attachment: 0,
								layout: ImageLayout::ColorAttachmentOptimal,
								..Default::default()
							})
						],
						..Default::default()
					}
				],
				..Default::default()
			}
		).map_err(|_| ())?;

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
		).map_err(|_| ())?;

		let mut framebuffers = Vec::with_capacity(images.len());
		for x in 0..images.len() {
			let output = ImageView::new_default(images[x].clone()).map_err(|_| ())?;

			framebuffers.push(Framebuffer::new(
				render_pass.clone(), 
				FramebufferCreateInfo {
					attachments: vec![
						output
					],
					extent: [window.size().0, window.size().1],
					..Default::default()
				},
			).map_err(|_| ())?);
		}
		let framebuffers = framebuffers.into_boxed_slice();

		let mut futures = Vec::with_capacity(images.len());
		for _ in 0..images.len() {
			futures.push(Operation { operation_type: OperationType::Graphics, future: None });
		}
		let futures = futures.into_boxed_slice();

		self.surfaces.insert(uuid, Box::new(WindowSurface {
			window:				window,
			render_pass:		render_pass,
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
