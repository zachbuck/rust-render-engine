
use std::sync::Arc;

use vulkano::{
	buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer}, 
	descriptor_set::{WriteDescriptorSet}, 
	image::{
		sampler::Sampler, 
		view::ImageView,
	}, 
	memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
};

use crate::{
	render_engine::render_resources::DefaultResources, 
	shader::descriptor_requirements::{DescriptorType, UniformBufferElement}, texture::Texture,
};

#[derive(Debug)]
pub enum DescriptorData {
	CombinedImageSampler(Arc<Texture>),
	UniformBuffer(Box<[u8]>),
}

#[derive(Debug)]
pub(crate) enum DescriptorDataInternal {
	CombinedImageSampler(Arc<ImageView>, Arc<Sampler>),
	UniformBuffer(Subbuffer<[u8]>),
}

impl DescriptorData {
	pub(crate) fn compatable_with(&self, descriptor_type: &DescriptorType) -> bool {
		match self {
			DescriptorData::CombinedImageSampler(_) => *descriptor_type == DescriptorType::CombinedImageSampler,
			DescriptorData::UniformBuffer(items) => {
				if let DescriptorType::UniformBuffer(uniforms) = descriptor_type {
					UniformBufferElement::size(uniforms) as usize == items.len()
				} else {
					false
				}
			}
		}
	}
}

impl DescriptorDataInternal {
	pub(crate) fn get_default(descriptor_type: &DescriptorType, default: &DefaultResources, allocator: &Arc<StandardMemoryAllocator>) -> Self {
		match descriptor_type {
			DescriptorType::CombinedImageSampler => {
				let texture = &default.texture;

				DescriptorDataInternal::CombinedImageSampler(texture.image.clone(), texture.sampler.clone())
			},

			DescriptorType::UniformBuffer(uniforms) => {
				let buffer: Subbuffer<[u8]> = Buffer::new_slice(
					allocator.clone(), 
					BufferCreateInfo {
						usage: BufferUsage::UNIFORM_BUFFER,
						..Default::default()
					}, 
					AllocationCreateInfo {
						memory_type_filter: MemoryTypeFilter::PREFER_DEVICE | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
						..Default::default()
					},
					UniformBufferElement::size(uniforms) as u64
				).unwrap();

				DescriptorDataInternal::UniformBuffer(buffer)
			},
		}
	}

	pub(crate) fn get_descriptor_write((binding, descriptor): &(u32, DescriptorDataInternal)) -> WriteDescriptorSet{
		let data = &descriptor;
		match data {
			DescriptorDataInternal::CombinedImageSampler(image_view, sampler) => {
				WriteDescriptorSet::image_view_sampler(*binding, image_view.clone(), sampler.clone())
			},
			DescriptorDataInternal::UniformBuffer(subbuffer) => {
				WriteDescriptorSet::buffer(*binding, subbuffer.clone())
			},
		}
	}
}