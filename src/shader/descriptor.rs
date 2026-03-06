
use std::sync::Arc;

use vulkano::{buffer::{Buffer, BufferCreateInfo, Subbuffer}, descriptor_set::WriteDescriptorSet, memory::allocator::{AllocationCreateInfo, StandardMemoryAllocator}, shader::DescriptorBindingRequirements};

use crate::shader::descriptor::uniform::{Uniform, UniformType};

pub mod uniform;

#[derive(Clone)]
#[derive(Debug)]
pub struct Descriptor {
	pub(crate) set: u32,
	pub(crate) binding: u32,
	pub(crate) count: u32,

	pub(crate) descriptor_type: DescriptorType,
}

impl Descriptor {
	pub fn uniform_buffer(set: u32, binding: u32, uniforms: &[UniformType]) -> Self {
		return Descriptor {
			set: set,
			binding: binding,
			count: 1,
			descriptor_type: DescriptorType::UniformBuffer(uniforms.to_vec()),
		};
	}

	pub(super) fn is_compatable_with_requirements(&self, requirements: &DescriptorBindingRequirements) -> bool {
		match self.descriptor_type {
			DescriptorType::UniformBuffer(_) => requirements.descriptor_types.contains(&vulkano::descriptor_set::layout::DescriptorType::UniformBuffer),
			DescriptorType::Unknown => true,
		}
	}

	pub(super) fn is_compatable_with_descriptor(&self, other: &Descriptor) -> bool {
		if self.descriptor_type != other.descriptor_type { return false; }
		if self.count != other.count { return false; }
		return true;
	}

	pub(super) fn from_requirements(set: u32, binding: u32, requirements: &DescriptorBindingRequirements) -> Self {
		if requirements.descriptor_types.contains(&vulkano::descriptor_set::layout::DescriptorType::UniformBuffer) {
			return Descriptor {
				set: set,
				binding: binding,
				count: 0,
				descriptor_type: DescriptorType::UniformBuffer(Vec::new()),
			};
		} else {
			return Descriptor {
				set: set,
				binding: binding,
				count: 0,
				descriptor_type: DescriptorType::Unknown,
			}
		}
	}
}

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq, Eq)]
pub(crate) enum DescriptorType {
	UniformBuffer(Vec<UniformType>),
	Unknown,
}

impl Into<vulkano::descriptor_set::layout::DescriptorType> for &DescriptorType {
	fn into(self) -> vulkano::descriptor_set::layout::DescriptorType {
		match self {
			DescriptorType::UniformBuffer(_) => vulkano::descriptor_set::layout::DescriptorType::UniformBuffer,
			DescriptorType::Unknown => unimplemented!(),
		}
	}
}

pub enum DescriptorData {
	UniformBuffer(Subbuffer<[u8]>),
	Unknown,
}

impl DescriptorData {
	pub(crate) fn from_descriptor(descriptor: &Descriptor, allocator: Arc<StandardMemoryAllocator>) -> Self {
		match &descriptor.descriptor_type {
			DescriptorType::UniformBuffer(uniform_types) => {
				let uniforms = uniform_types.iter().map(|u| Uniform::from_type(u)).collect::<Vec<_>>();
				let data = Uniform::into_binary(&uniforms);

				let buffer = Buffer::from_iter(
					allocator,
					BufferCreateInfo {
						usage: vulkano::buffer::BufferUsage::UNIFORM_BUFFER,
						..Default::default()
					},
					AllocationCreateInfo {
						memory_type_filter: vulkano::memory::allocator::MemoryTypeFilter::PREFER_DEVICE | vulkano::memory::allocator::MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
						..Default::default()
					},
					data
				).unwrap();

				DescriptorData::UniformBuffer(buffer)
			},
			DescriptorType::Unknown => DescriptorData::Unknown,
		}
	}

	pub(crate) fn to_descriptor_write(&self, binding: u32) -> WriteDescriptorSet {
		match self {
			DescriptorData::UniformBuffer(subbuffer) => WriteDescriptorSet::buffer(binding, subbuffer.clone()),
			DescriptorData::Unknown => WriteDescriptorSet::none(binding),
		}
	}
}