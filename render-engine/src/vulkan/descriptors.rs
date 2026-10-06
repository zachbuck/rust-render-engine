
use std::sync::Arc;

use parsing::spir_v::{data_type::DataType, shader::DescriptorBinding};
use uuid::Uuid;
use vulkano::{
	buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer}, 
	descriptor_set::{
		DescriptorBufferInfo, 
		DescriptorSet as VulkanDescriptorSet, 
		WriteDescriptorSet, 
	}, 
	memory::allocator::{AllocationCreateInfo, MemoryTypeFilter}
};

use crate::{
	macros::debug_error, 
	vulkan::render_thread::RenderThread,
};

/* TODO
- [ ] Add Image support
- [ ] Add writeable (by the shader) uniform/image support
*/
pub struct DescriptorSet {
	set: u32,
	types: Vec<DescriptorBinding>,

	internal: Arc<VulkanDescriptorSet>,
	buffer: Subbuffer<[u8]>,
}

impl RenderThread {
	fn create_descriptor_set(&mut self, set: u32, pipeline: Uuid) -> Result<(Uuid,), ()> {
		let uuid = Uuid::now_v7();

		let shader_collection = self.pipelines.get(&pipeline).unwrap();
		let set_index = shader_collection.descriptor_layouts.binary_search_by_key(&set, |(set, _)| *set).map_err(debug_error!())?;
		let (_, descriptor_set_layout) = &shader_collection.descriptor_layouts[set_index];
		let descriptor_set_types = &shader_collection.descriptors.descriptors[set_index];

		let buffer = Buffer::new_slice(
			self.buffer_allocator.clone(), 
			BufferCreateInfo {
				usage: BufferUsage::UNIFORM_BUFFER,
				..Default::default()
			}, 
			AllocationCreateInfo {
				memory_type_filter: MemoryTypeFilter::HOST_RANDOM_ACCESS,
				..Default::default()
			}, 
			descriptor_set_types.get_size() as u64,
		).map_err(debug_error!())?;

		let mut descriptor_writes = Vec::with_capacity(descriptor_set_types.bindings.len());
		let mut index = 0u64;
		for binding in &descriptor_set_types.bindings {
			descriptor_writes.push(match binding.data_type {
				DataType::Struct { members: _ } => {
					let data_size = binding.data_type.get_size() as u64;
					let write = WriteDescriptorSet::buffer_with_range(
						binding.binding, 
						DescriptorBufferInfo {
							buffer: buffer.clone(),
							range: index..(index+data_size),
						},
					);
					index += data_size;
					write
				}
				DataType::Image { dimension: _, pixel_format: _, texture_format: _ } => todo!(),
				DataType::Sampler => todo!(),
				DataType::ImageSampler { dimension: _, pixel_format: _, texture_format: _ } => todo!(),
				_ => todo!(),
			});
		}

		let internal = VulkanDescriptorSet::new(
			self.descriptor_allocator.clone(), 
			descriptor_set_layout.clone(), 
			descriptor_writes, 
			vec![],
		).map_err(debug_error!())?;

		let descriptor_set = DescriptorSet {
			set,
			internal,
			types: descriptor_set_types.bindings.clone(),
			buffer,
		};

		self.descriptor_sets.insert(uuid, descriptor_set);

		Ok((uuid,))
	}

	fn set_descriptor_set_binding(&mut self, descriptor_set: Uuid, binding: u32, data: Box<[u8]>) -> Result<(), ()> {
		let descriptor_set = self.descriptor_sets.get_mut(&descriptor_set).unwrap();

		let mut offset = 0;
		let mut descriptor_size = None;
		for descriptor_binding in &descriptor_set.types {
			if descriptor_binding.binding == binding { 
				descriptor_size = Some(descriptor_binding.data_type.get_size());
				break; 
			}
			offset += descriptor_binding.data_type.get_size();
		}
		let descriptor_size = descriptor_size.ok_or(())?;

		let mut buffer = descriptor_set.buffer.write().map_err(debug_error!())?;
		buffer[offset..(offset+descriptor_size)].copy_from_slice(&data);

		Ok(())
	}

	fn drop_descriptor_set(&mut self, uuid: Uuid) {
		self.descriptor_sets.remove(&uuid);
	}
}
