
use std::{
	collections::{BTreeMap, HashMap}, 
	sync::Arc
};

use vulkano::{
	descriptor_set::{
		layout::{DescriptorBindingFlags, DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutCreateFlags, DescriptorSetLayoutCreateInfo}
	}, 
	device::Device, 
	shader::EntryPoint
};

use crate::{
	macros::error_map, 
	shader::{SHADER_TYPES, ShaderType},
};

#[derive(Debug)]
#[derive(Clone)]
pub(crate) struct DescriptorRequirements {
	pub(crate) sets: Box<[DescriptorSetRequirements]>,
}

#[derive(Debug)]
#[derive(Clone)]
pub(crate) struct DescriptorSetRequirements {
	pub(crate) set: u32,

	pub(crate) bindings: Box<[DescriptorBindingRequirements]>,
}

#[derive(Debug)]
#[derive(Clone, Copy)]
pub(crate) struct DescriptorBindingRequirements {
	pub(crate) binding: u32,

	pub(crate) descriptor_type: DescriptorType,
	stages: ShaderStages,
}

#[derive(Debug)]
#[derive(Clone, Copy)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum DescriptorType {
	CombinedImageSampler,
}

#[derive(Debug)]
#[derive(Clone, Copy)]
pub(crate) struct ShaderStages {
	stages: u8,
}

impl DescriptorRequirements {
	pub(crate) fn empty() -> Self {
		DescriptorRequirements {
			sets: Box::new([]),
		}
	}

	pub(crate) fn from_vulkano(entry_point: &EntryPoint) -> Result<Self, ()> {
		let requirement_set = &entry_point.info().descriptor_binding_requirements;

		let mut stage_one = HashMap::new();
		for ((set, binding), requirements) in requirement_set {
			if !stage_one.contains_key(set) {
				stage_one.insert(*set, Vec::new());
			}
			let set = stage_one.get_mut(set).unwrap();

			if requirements.descriptor_types.contains(&vulkano::descriptor_set::layout::DescriptorType::CombinedImageSampler) {
				if requirements.descriptor_count != Some(1) { return Err(()) }
				if requirements.image_view_type != Some(vulkano::image::view::ImageViewType::Dim2d) { return Err(()) }

				set.push(DescriptorBindingRequirements {
					binding: *binding,

					descriptor_type: DescriptorType::CombinedImageSampler,
					stages: ShaderStages::from_shader_type(entry_point.info().execution_model.into()),
				});
			} else {
				todo!("Only Combined Image Sampler descriptors are currently supported.")
			}
		}

		let mut stage_two = Vec::new();
		for (set, mut bindings) in stage_one {
			bindings.sort_by_key(|bind| bind.binding);

			let set = DescriptorSetRequirements {
				set: set,
				bindings: bindings.into_boxed_slice(),
			};

			stage_two.push(set);
		}

		stage_two.sort_by_key(|set| set.set);

		Ok(DescriptorRequirements {
			sets: stage_two.into_boxed_slice(),
		})
	}

	pub(crate) fn merge_with(self, other: &DescriptorRequirements) -> Self {
		let mut stage_one = HashMap::new();

		for set in &self.sets {
			stage_one.insert(set.set, HashMap::new());

			let set_requirement = stage_one.get_mut(&set.set).unwrap();
			for binding in &set.bindings {
				set_requirement.insert(binding.binding, *binding);
			}
		}

		for set in &other.sets {
			if !stage_one.contains_key(&set.set) {
				stage_one.insert(set.set, HashMap::new());
			}

			let set_requirement = stage_one.get_mut(&set.set).unwrap();
			for binding in &set.bindings {
				if !set_requirement.contains_key(&binding.binding) {
					set_requirement.insert(binding.binding, *binding);
					continue
				} 

				let binding_requirement = set_requirement.get_mut(&binding.binding).unwrap();
				binding_requirement.stages.merge_with(&binding.stages);
			}
		}

		let mut stage_two = Vec::new();
		for (set, bindings) in stage_one {
			let mut bindings = bindings.values().map(|b| *b).collect::<Vec<_>>();
			bindings.sort_by_key(|b| b.binding);

			let set_requirement = DescriptorSetRequirements {
				set: set,
				bindings: bindings.into_boxed_slice(),
			};

			stage_two.push(set_requirement);
		}

		stage_two.sort_by_key(|s| s.set);

		DescriptorRequirements {
			sets: stage_two.into_boxed_slice(),
		}
	}
	
	pub(crate) fn test_compatibility(requirements: &[&DescriptorRequirements]) -> bool {
		let mut max_set = None;
		for requirement in requirements {
			if requirement.sets.len() == 0 { continue }
			let set = requirement.sets.last().unwrap().set;
			if max_set.is_none() { max_set = Some(set); }
			if set > max_set.unwrap() { max_set = Some(set); }
		}

		if max_set.is_none() { return true; }

		for set in 0..=max_set.unwrap() {
			let set_requirements = requirements.iter().filter_map(|r| r.sets.iter().find(|s| s.set == set)).collect::<Vec<_>>();

			let set_compatible = Self::test_set_compatibility(&set_requirements);
			if !set_compatible { return false; }
		}

		return true;
	}

	fn test_set_compatibility(requirements: &[&DescriptorSetRequirements]) -> bool {
		let mut max_binding = None;
		for requirement in requirements {
			if requirement.bindings.len() == 0 { continue }
			let binding = requirement.bindings.last().unwrap().binding;
			if max_binding.is_none() { max_binding = Some(binding); }
			if binding > max_binding.unwrap() { max_binding = Some(binding); }
		}

		if max_binding.is_none() { return true; }

		for binding in 0..=max_binding.unwrap() {
			let binding_requirements = requirements.iter().filter_map(|s| s.bindings.iter().find(|b| b.binding == binding)).collect::<Vec<_>>();

			let binding_compatible = Self::test_binding_compatibility(&binding_requirements);
			if !binding_compatible { return false; }
		}

		return true;
	}

	fn test_binding_compatibility(requirements: &[&DescriptorBindingRequirements]) -> bool {
		if requirements.len() <= 1 { return true; }

		let first = requirements.first().unwrap();
		for i in 1..=requirements.len() {
			if first.descriptor_type != requirements[i].descriptor_type { return false; }
		}

		return true;
	}

	pub(crate) fn get_descriptor_layout(&self, device: &Arc<Device>) -> Result<Vec<Arc<DescriptorSetLayout>>, ()> {
		let mut out = Vec::new();

		for set in &self.sets {
			let mut bindings = BTreeMap::new();

			for binding in &set.bindings {
				let binding_layout = DescriptorSetLayoutBinding {
					binding_flags: DescriptorBindingFlags::empty(),
					descriptor_count: 1,
					stages: binding.stages.as_vulkano_shader_stages(),
					immutable_samplers: Vec::new(),
					..DescriptorSetLayoutBinding::descriptor_type(binding.descriptor_type.as_vulkano_descriptor_type())
				};

				bindings.insert(binding.binding, binding_layout);
			}

			let set_layout = DescriptorSetLayout::new(
				device.clone(), 
				DescriptorSetLayoutCreateInfo {
					flags: DescriptorSetLayoutCreateFlags::empty(),
					bindings: bindings,
					..Default::default()
				}
			).map_err(error_map!())?;

			out.push(set_layout);
		}

		return Ok(out);
	}
}

impl ShaderStages {
	fn from_shader_type(shader_type: ShaderType) -> Self {
		ShaderStages {
			stages: shader_type as u8,
		}
	}

	fn merge_with(&mut self, other: &ShaderStages) {
		self.stages &= other.stages;
	}

	fn as_vulkano_shader_stages(&self) -> vulkano::shader::ShaderStages {
		let mut out = vulkano::shader::ShaderStages::empty();

		for shader_type in SHADER_TYPES {
			if shader_type as u8 & self.stages != 0 {
				out = out.union(shader_type.into());
			}
		}

		return out;
	}
}

impl DescriptorType {
	fn as_vulkano_descriptor_type(&self) -> vulkano::descriptor_set::layout::DescriptorType {
		match self {
			DescriptorType::CombinedImageSampler => vulkano::descriptor_set::layout::DescriptorType::CombinedImageSampler,
		}
	}
}
