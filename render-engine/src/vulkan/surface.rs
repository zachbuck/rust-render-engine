
use std::sync::Arc;

use vulkano::{
	command_buffer::allocator::StandardCommandBufferAllocator, 
	device::Queue,
};

use crate::vulkan::render_thread::Operation;

pub mod window_surface;

pub trait Surface {
	fn begin_rendering(&mut self, allocator: &Arc<StandardCommandBufferAllocator>, graphics_queue: &Arc<Queue>) -> Result<(), ()>;
	fn end_rendering(&mut self, graphics_queue: &Arc<Queue>, previous_operation: Operation) -> Result<Operation, ()>;
}
