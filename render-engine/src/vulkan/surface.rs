
use std::sync::Arc;

use uuid::Uuid;
use vulkano::{
	command_buffer::allocator::StandardCommandBufferAllocator, 
	device::Queue, 
	format::Format, 
	image::ImageLayout, 
	render_pass::{AttachmentDescription, AttachmentLoadOp, AttachmentReference, AttachmentStoreOp, RenderPass, RenderPassCreateInfo, SubpassDescription},
};

use crate::{
	macros::error_to_unit_type, surface::RenderPassCreateInfo as RenderPassInfo, vulkan::render_thread::{Operation, RenderThread},
};

pub mod window_surface;

pub trait Surface {
	fn begin_rendering(&mut self, allocator: &Arc<StandardCommandBufferAllocator>, graphics_queue: &Arc<Queue>) -> Result<(), ()>;
	fn end_rendering(&mut self, graphics_queue: &Arc<Queue>, previous_operation: Operation) -> Result<Operation, ()>;

	fn get_renderpass(&self) -> &Uuid;
}

impl RenderThread {
	pub fn get_renderpass(&mut self, _render_pass_info: RenderPassInfo) -> Result<Uuid, ()> {
		for (key, _) in &self.render_passes {
			return Ok(*key);
		}

		let uuid = Uuid::now_v7();

		let renderpass = RenderPass::new(
			self.device.clone(), 
			RenderPassCreateInfo {
				attachments: 	vec![
					AttachmentDescription {
						format: 		Format::R8G8B8A8_UNORM,
						load_op: 		AttachmentLoadOp::Clear,
						store_op: 		AttachmentStoreOp::Store,
						final_layout: 	ImageLayout::ColorAttachmentOptimal,
						..Default::default()
					},
				],
				subpasses: 		vec![
					SubpassDescription {
						input_attachments: 	Vec::new(),
						color_attachments: 	vec![
							Some(AttachmentReference {
								attachment: 0,
								layout: 	ImageLayout::ColorAttachmentOptimal,
								..Default::default()
							})
						],
						..Default::default()
					},
				],
				..Default::default()
			}
		).map_err(error_to_unit_type!())?;

		self.render_passes.insert(uuid, renderpass);

		Ok(uuid)
	}
}
