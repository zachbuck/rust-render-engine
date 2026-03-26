use std::{
	collections::HashMap, 
	sync::{Arc, Weak}
};

use uuid::Uuid;
use vulkano::{
	buffer::{Buffer, BufferCreateInfo, BufferUsage}, 
	command_buffer::{
		AutoCommandBufferBuilder, 
		CommandBufferUsage, 
		CopyBufferToImageInfo, 
		PrimaryCommandBufferAbstract, 
		allocator::StandardCommandBufferAllocator
	}, 
	device::{Device, Queue}, 
	format::Format, 
	image::{
		Image, 
		ImageCreateInfo, 
		ImageType, 
		ImageUsage, 
		sampler::{Filter, Sampler, SamplerCreateInfo}, 
		view::ImageView
	}, 
	memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator}, 
	sync::GpuFuture
};

use crate::{
	macros::error_map, mesh_data::{
		MeshData, 
		mesh_data_internal::MeshDataInternal,
	}, pipeline::{
		Pipeline, 
		pipeline_internal::PipelineInternal,
	}, texture::{
		Texture, 
		texture_internal::TextureInternal,
	}
};

#[derive(Debug)]
pub(crate) struct RenderResources<'a> {
	mesh_data: 	&'a HashMap<Uuid, MeshDataInternal>,
	pipelines: 	&'a HashMap<Uuid, PipelineInternal>,
	textures: 	&'a HashMap<Uuid, TextureInternal>,
}

impl<'a> RenderResources<'a> {
	#[inline]
	pub(crate) fn new(mesh_data: &'a HashMap<Uuid, MeshDataInternal>, pipelines: &'a HashMap<Uuid, PipelineInternal>, textures: &'a HashMap<Uuid, TextureInternal>) -> Self {
		RenderResources { 
			mesh_data, 
			pipelines, 
			textures, 
		}
	}
}

impl RenderResources<'_> {
	#[inline]
	pub(crate) fn get_mesh_data(&self, reference: &Arc<MeshData>) -> Option<&MeshDataInternal> {
		self.mesh_data.get(&reference.uuid)
	}

	#[inline]
	pub(crate) fn get_pipeline(&self, reference: &Arc<Pipeline>) -> Option<&PipelineInternal> {
		self.pipelines.get(&reference.uuid)
	}

	#[inline]
	pub(crate) fn get_texture(&self, reference: &Arc<Texture>) -> Option<&TextureInternal> {
		self.textures.get(&reference.uuid)
	}
}

pub(crate) struct DefaultResources {
	pub(crate) texture: TextureInternal,
}

impl DefaultResources {
	pub(crate) fn generate(device: &Arc<Device>, queue: &Arc<Queue>, buffer_allocator: &Arc<StandardMemoryAllocator>, command_allocator: &Arc<StandardCommandBufferAllocator>) -> Result<Self, ()> {
		let texture_data = [
				255u8, 0u8, 0u8, 255u8,
				0u8, 255u8, 0u8, 255u8,
				0u8, 0u8, 255u8, 255u8,
				128u8, 128u8, 128u8, 255u8,
			];
		
		let buffer = Buffer::from_iter(
			buffer_allocator.clone(), 
			BufferCreateInfo {
				usage: BufferUsage::TRANSFER_SRC,
				..Default::default()
			}, 
			AllocationCreateInfo {
				memory_type_filter: MemoryTypeFilter::HOST_SEQUENTIAL_WRITE | MemoryTypeFilter::PREFER_HOST,
				..Default::default()
			}, 
			texture_data,
		).map_err(error_map!())?;

		let image = Image::new(
			buffer_allocator.clone(), 
			ImageCreateInfo {
				image_type: ImageType::Dim2d,
				format: Format::R8G8B8A8_UNORM,
				extent: [2, 2, 1],
				usage: ImageUsage::SAMPLED | ImageUsage::TRANSFER_DST,
				..Default::default()
			}, 
			AllocationCreateInfo {
				memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
				..Default::default()
			}
		).map_err(error_map!())?;

		let mut builder = AutoCommandBufferBuilder::primary(
			command_allocator.clone(), 
			queue.queue_family_index(), 
			CommandBufferUsage::OneTimeSubmit,
		).map_err(error_map!())?;

		builder
			.copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(buffer, image.clone())).map_err(error_map!())?;

		let commands = builder.build().map_err(error_map!())?;

		commands
			.execute(queue.clone()).map_err(error_map!())?
			.then_signal_fence_and_flush().map_err(error_map!())?
			.wait(None).map_err(error_map!())?;

		let image_view = ImageView::new_default(image.clone()).map_err(error_map!())?;

		let sampler = Sampler::new(
			device.clone(), 
			SamplerCreateInfo {
				mag_filter: Filter::Nearest,
				min_filter: Filter::Nearest,
				..Default::default()
			}
		).map_err(error_map!())?;

		let texture = TextureInternal {
			reference: Weak::new(),
			image: image_view,
			sampler: sampler,
		};

		Ok(DefaultResources {
			texture: texture,
		})
	}
}