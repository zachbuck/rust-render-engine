
use std::sync::Arc;

use vulkano::{
	command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer}, 
	descriptor_set::{
		DescriptorSet, 
		WriteDescriptorSet, 
		allocator::StandardDescriptorSetAllocator, 
		layout::DescriptorSetLayout,
	}, 
	pipeline::{PipelineBindPoint, PipelineLayout},
};

use crate::{
	macros::error_map, render_engine::render_resources::{DefaultResources, RenderResources}, shader::descriptor_requirements::{DescriptorBindingRequirements, DescriptorSetRequirements, DescriptorType}, texture::Texture
};

#[derive(Debug)]
pub(crate) struct DescriptorSetData {
	pub(crate) set: u32,
	descriptor_set: Arc<DescriptorSet>,
	bindings: Box<[DescriptorBindingData]>,
}

#[derive(Debug)]
pub(crate) struct DescriptorBindingData {
	binding: u32,
	write_data: WriteDescriptorSet,
}

#[derive(Debug)]
pub enum DescriptorData {
	CombinedImageSampler(Arc<Texture>)
}

impl DescriptorSetData {
	pub(crate) fn bind<'a>(&self, builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, layout: &Arc<PipelineLayout>) -> Result<&'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, ()> {
		builder
			.bind_descriptor_sets(
				PipelineBindPoint::Graphics, 
				layout.clone(), 
				self.set, 
				self.descriptor_set.clone(),
			).map_err(error_map!())
	}

	pub(crate) fn new(requirements: &DescriptorSetRequirements, allocator: &Arc<StandardDescriptorSetAllocator>, layout: &Arc<DescriptorSetLayout>, defaults: &DefaultResources) -> Result<Self, ()> {
		let mut bindings = Vec::with_capacity(requirements.bindings.len());
		for binding in &requirements.bindings {
			bindings.push(DescriptorBindingData::new(binding, defaults));
		}
		let bindings = bindings.into_boxed_slice();

		let descriptor_set = DescriptorSet::new(
			allocator.clone(), 
			layout.clone(), 
			bindings.iter().map(|b| b.write_data.clone()), 
			[],
		).map_err(error_map!())?;

		Ok(DescriptorSetData {
			set: requirements.set,
			descriptor_set: descriptor_set,
			bindings: bindings,
		})
	}

	pub(crate) fn update_binding(&mut self, binding: u32, data: DescriptorData, render_resources: &RenderResources) -> Result<(), ()> {
		let binding = self.bindings.iter_mut().find(|b| b.binding == binding).unwrap();
		binding.write_data = data.to_write(binding.binding, render_resources);
		unsafe {
			self.descriptor_set.update_by_ref(
				self.bindings.iter().map(|b| b.write_data.clone()), 
				[]
			).map_err(error_map!())?;
		}
		Ok(())
	}
}

impl DescriptorBindingData {
	pub(crate) fn new(requirements: &DescriptorBindingRequirements, defaults: &DefaultResources) -> Self {
		let write_data = 
		match requirements.descriptor_type {
			DescriptorType::CombinedImageSampler 	=> WriteDescriptorSet::image_view_sampler(requirements.binding, defaults.texture.image.clone(), defaults.texture.sampler.clone()),
		};

		DescriptorBindingData {
			binding: requirements.binding,
			write_data: write_data,
		}
	}
}

impl DescriptorData {
	pub(crate) fn to_write(&self, binding: u32, render_resources: &RenderResources) -> WriteDescriptorSet {
		match self {
			DescriptorData::CombinedImageSampler(reference) => {
				let texture_internal = render_resources.get_texture(reference).unwrap();
				WriteDescriptorSet::image_view_sampler(
					binding, 
					texture_internal.image.clone(),
					texture_internal.sampler.clone(),
				)
			}
		}
	}

	pub(crate) fn comptable_with_type(&self, descriptor_type: DescriptorType) -> bool {
		match self {
			DescriptorData::CombinedImageSampler(_) => descriptor_type == DescriptorType::CombinedImageSampler,
		}
	}
}
