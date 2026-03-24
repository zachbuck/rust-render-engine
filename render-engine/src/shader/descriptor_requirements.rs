
use std::collections::HashMap;

use vulkano::shader::EntryPoint;

#[derive(Debug)]
pub(crate) struct DescriptorRequirements {
	sets: Box<[DescriptorSetRequirements]>,
}

#[derive(Debug)]
#[derive(Clone)]
pub(crate) struct DescriptorSetRequirements {
	set: u32,
	bindings: Box<[DescriptorBindingRequirements]>,
}

#[derive(Debug)]
#[derive(Clone, Copy)]
pub(crate) struct DescriptorBindingRequirements {
	binding: u32,
	descriptor_type: DescriptorType,
}

#[derive(Debug)]
#[derive(Clone, Copy)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum DescriptorType {
	CombinedImageSampler,
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
				});
			} else {
				return Err(())
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
}
