
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
pub(crate) struct DescriptorDataInternal {
	pub(crate) descriptor_data: DescriptorDataType,
}

#[derive(Debug)]
pub(crate) enum DescriptorDataType {
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

				let data_type = DescriptorDataType::CombinedImageSampler(
					texture.image.clone(), 
					texture.sampler.clone()
				);

				DescriptorDataInternal { descriptor_data: data_type }
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

				let data_type = DescriptorDataType::UniformBuffer(
					buffer,
				);

				DescriptorDataInternal { descriptor_data: data_type }
			},
		}
	}

	pub(crate) fn get_descriptor_write((binding, descriptor): &(u32, DescriptorDataInternal)) -> WriteDescriptorSet{
		let data = &descriptor.descriptor_data;
		match data {
			DescriptorDataType::CombinedImageSampler(image_view, sampler) => {
				WriteDescriptorSet::image_view_sampler(*binding, image_view.clone(), sampler.clone())
			},
			DescriptorDataType::UniformBuffer(subbuffer) => {
				WriteDescriptorSet::buffer(*binding, subbuffer.clone())
			},
		}
	}
}