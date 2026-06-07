
pub trait PixelFormat {
	const VULKAN_FORMAT: vulkano::format::Format;
}

#[repr(C)]
pub struct RGBA8 {
	pub r: u8,
	pub g: u8,
	pub b: u8,
	pub a: u8,
}

impl PixelFormat for RGBA8 {
	const VULKAN_FORMAT: vulkano::format::Format = vulkano::format::Format::R8G8B8A8_UNORM;
}