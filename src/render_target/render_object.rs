
use std::sync::Arc;

use uuid::Uuid;
use vulkano::{
	buffer::{Buffer, BufferCreateInfo, Subbuffer}, command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer}, descriptor_set::WriteDescriptorSet, memory::allocator::{AllocationCreateInfo, StandardMemoryAllocator}, pipeline::Pipeline
};

use crate::{
	RenderEngine, 
	mesh_data::{MeshData, MeshDataInternal}, 
	render_target::{RenderCall, RenderTarget}, 
	shader::{Descriptor, DescriptorType, GraphicsProgram, GraphicsProgramInternal, UniformType}, 
	unwrap_option_or_none
};

pub struct RenderObjectHandle {
	uuid: Uuid,	
}

enum DescriptorData {
	UniformBuffer(Subbuffer<[u8]>),
	Unknown,
}

impl DescriptorData {
	fn from_descriptor(descriptor: &Descriptor, allocator: Arc<StandardMemoryAllocator>) -> Self {
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

	fn to_descriptor_write(&self, binding: u32) -> WriteDescriptorSet {
		match self {
			DescriptorData::UniformBuffer(subbuffer) => WriteDescriptorSet::buffer(binding, subbuffer.clone()),
			DescriptorData::Unknown => WriteDescriptorSet::none(binding),
		}
	}
}

enum Uniform {
	Float	(f32),
	Vec2	([f32; 2]),
	Vec3	([f32; 3]),
	Vec4	([f32; 4]),
	Mat2	([[f32; 2]; 2]),
	Mat2x3	([[f32; 2]; 3]),
	Mat2x4	([[f32; 2]; 4]),
	Mat3	([[f32; 3]; 3]),
	Mat3x2	([[f32; 3]; 2]),
	Mat3x4	([[f32; 3]; 4]),
	Mat4	([[f32; 4]; 4]),
	Mat4x2	([[f32; 4]; 2]),
	Mat4x3	([[f32; 4]; 3]),

	Double	(f64),
	DVec2	([f64; 2]),
	DVec3	([f64; 3]),
	DVec4	([f64; 4]),
	DMat2	([[f64; 2]; 2]),
	DMat2x3	([[f64; 2]; 3]),
	DMat2x4	([[f64; 2]; 4]),
	DMat3	([[f64; 3]; 3]),
	DMat3x2	([[f64; 3]; 2]),
	DMat3x4	([[f64; 3]; 4]),
	DMat4	([[f64; 4]; 4]),
	DMat4x2	([[f64; 4]; 2]),
	DMat4x3	([[f64; 4]; 3]),

	Int		(i32),
	IVec2	([i32; 2]),
	IVec3	([i32; 3]),
	IVec4	([i32; 4]),
	IMat2	([[i32; 2]; 2]),
	IMat2x3	([[i32; 2]; 3]),
	IMat2x4	([[i32; 2]; 4]),
	IMat3	([[i32; 3]; 3]),
	IMat3x2	([[i32; 3]; 2]),
	IMat3x4	([[i32; 3]; 4]),
	IMat4	([[i32; 4]; 4]),
	IMat4x2	([[i32; 4]; 2]),
	IMat4x3	([[i32; 4]; 3]),

	UInt	(u32),
	UVec2	([u32; 2]),
	UVec3	([u32; 3]),
	UVec4	([u32; 4]),
	UMat2	([[u32; 2]; 2]),
	UMat2x3	([[u32; 2]; 3]),
	UMat2x4	([[u32; 2]; 4]),
	UMat3	([[u32; 3]; 3]),
	UMat3x2	([[u32; 3]; 2]),
	UMat3x4	([[u32; 3]; 4]),
	UMat4	([[u32; 4]; 4]),
	UMat4x2	([[u32; 4]; 2]),
	UMat4x3	([[u32; 4]; 3]),
}

macro_rules! scalar_to_binary {
	($data: ident, $binary:ident, $to_binary: ident) => {
		{$binary.append(&mut $data.$to_binary().to_vec());}
	}
}

macro_rules! vec_to_binary {
	($data: ident, $binary:ident, $to_binary:ident) => {
		for element in $data {
			$binary.append(&mut element.$to_binary().to_vec());
		}
	}
}

macro_rules! mat_to_binary {
	($data: ident, $binary:ident, $to_binary:ident) => {
		for row in $data {
			for element in row {
				$binary.append(&mut element.$to_binary().to_vec());
			}
		}
	}
}

impl Uniform {
	fn from_type(uniform_type: &UniformType) -> Self {
		match uniform_type {
			UniformType::Float => Uniform::Float(
							1.0
			),
			UniformType::Vec2 => Uniform::Vec2(
							[1.0, 1.0]
			),
			UniformType::Vec3 => Uniform::Vec3(
							[1.0, 1.0, 1.0]
			),
			UniformType::Vec4 => Uniform::Vec4(
							[1.0, 1.0, 1.0, 1.0]
			),
			UniformType::Mat2 => Uniform::Mat2(
							[[1.0, 0.0],
							[0.0, 1.0]]
			),
			UniformType::Mat2x3 => Uniform::Mat2x3(
							[[1.0, 0.0],
							[0.0, 1.0],
							[0.0, 0.0]]
			),
			UniformType::Mat2x4 => Uniform::Mat2x4(
							[[1.0, 0.0],
							[0.0, 1.0],
							[0.0, 0.0],
							[0.0, 0.0]]
			),
			UniformType::Mat3 => Uniform::Mat3(
							[[1.0, 0.0, 0.0],
							[0.0, 1.0, 0.0],
							[0.0, 0.0, 1.0]]
			),
			UniformType::Mat3x2 => Uniform::Mat3x2(
							[[1.0, 0.0, 0.0],
							[0.0, 1.0, 0.0]]
			),
			UniformType::Mat3x4 => Uniform::Mat3x4(
							[[1.0, 0.0, 0.0],
							[0.0, 1.0, 0.0],
							[0.0, 0.0, 1.0],
							[0.0, 0.0, 0.0]]
			),
			UniformType::Mat4 => Uniform::Mat4(
							[[1.0, 0.0, 0.0, 0.0],
							[0.0, 1.0, 0.0, 0.0],
							[0.0, 0.0, 1.0, 0.0],
							[0.0, 0.0, 0.0, 1.0]]
			),
			UniformType::Mat4x2 => Uniform::Mat4x2(
							[[1.0, 0.0, 0.0, 0.0],
							[0.0, 1.0, 0.0, 0.0]]
			),
			UniformType::Mat4x3 => Uniform::Mat4x3(
							[[1.0, 0.0, 0.0, 0.0],
							[0.0, 1.0, 0.0, 0.0],
							[0.0, 0.0, 1.0, 0.0]]
			),

			UniformType::Double => Uniform::Double(
							1.0
			),
			UniformType::DVec2 => Uniform::DVec2(
							[1.0, 1.0]
			),
			UniformType::DVec3 => Uniform::DVec3(
							[1.0, 1.0, 1.0]
			),
			UniformType::DVec4 => Uniform::DVec4(
							[1.0, 1.0, 1.0, 1.0]
			),
			UniformType::DMat2 => Uniform::DMat2(
							[[1.0, 0.0],
							[0.0, 1.0]]
			),
			UniformType::DMat2x3 => Uniform::DMat2x3(
							[[1.0, 0.0],
							[0.0, 1.0],
							[0.0, 0.0]]
			),
			UniformType::DMat2x4 => Uniform::DMat2x4(
							[[1.0, 0.0],
							[0.0, 1.0],
							[0.0, 0.0],
							[0.0, 0.0]]
			),
			UniformType::DMat3 => Uniform::DMat3(
							[[1.0, 0.0, 0.0],
							[0.0, 1.0, 0.0],
							[0.0, 0.0, 1.0]]
			),
			UniformType::DMat3x2 => Uniform::DMat3x2(
							[[1.0, 0.0, 0.0],
							[0.0, 1.0, 0.0]]
			),
			UniformType::DMat3x4 => Uniform::DMat3x4(
							[[1.0, 0.0, 0.0],
							[0.0, 1.0, 0.0],
							[0.0, 0.0, 1.0],
							[0.0, 0.0, 0.0]]
			),
			UniformType::DMat4 => Uniform::DMat4(
							[[1.0, 0.0, 0.0, 0.0],
							[0.0, 1.0, 0.0, 0.0],
							[0.0, 0.0, 1.0, 0.0],
							[0.0, 0.0, 0.0, 1.0]]
			),
			UniformType::DMat4x2 => Uniform::DMat4x2(
							[[1.0, 0.0, 0.0, 0.0],
							[0.0, 1.0, 0.0, 0.0]]
			),
			UniformType::DMat4x3 => Uniform::DMat4x3(
							[[1.0, 0.0, 0.0, 0.0],
							[0.0, 1.0, 0.0, 0.0],
							[0.0, 0.0, 1.0, 0.0]]
			),

			UniformType::Int => Uniform::Int(
				1
			),
			UniformType::IVec2 => Uniform::IVec2(
				[1, 1]
			),
			UniformType::IVec3 => Uniform::IVec3(
				[1, 1, 1]
			),
			UniformType::IVec4 => Uniform::IVec4(
				[1, 1, 1, 1]
			),
			UniformType::IMat2 => Uniform::IMat2(
				[[1, 0],
				 [0, 1]]
			),
			UniformType::IMat2x3 => Uniform::IMat2x3(
				[[1, 0],
				 [0, 1],
				 [0, 0]]
			),
			UniformType::IMat2x4 => Uniform::IMat2x4(
				[[1, 0],
				 [0, 1],
				 [0, 0],
				 [0, 0]]
			),
			UniformType::IMat3 => Uniform::IMat3(
				[[1, 0, 0], 
				 [0, 1, 0], 
				 [0, 0, 1]]
			),
			UniformType::IMat3x2 => Uniform::IMat3x2(
				[[1, 0, 0], 
				 [0, 1, 0]]
			),
			UniformType::IMat3x4 => Uniform::IMat3x4(
				[[1, 0, 0],
				 [0, 1, 0],
				 [0, 0, 1],
				 [0, 0, 0]]
			),
			UniformType::IMat4 => Uniform::IMat4(
				[[1, 0, 0, 0],
				 [0, 1, 0, 0],
				 [0, 0, 1, 0],
				 [0, 0, 0, 1]]
			),
			UniformType::IMat4x2 => Uniform::IMat4x2(
				[[1, 0, 0, 0],
				 [0, 1, 0, 0]]
			),
			UniformType::IMat4x3 => Uniform::IMat4x3(
				[[1, 0, 0, 0],
				 [0, 1, 0, 0],
				 [0, 0, 1, 0]]
			),

			UniformType::UInt => Uniform::UInt(
				1
			),
			UniformType::UVec2 => Uniform::UVec2(
				[1, 1]
			),
			UniformType::UVec3 => Uniform::UVec3(
				[1, 1, 1]
			),
			UniformType::UVec4 => Uniform::UVec4(
				[1, 1, 1, 1]
			),
			UniformType::UMat2 => Uniform::UMat2(
				[[1, 0],
				 [0, 1]]
			),
			UniformType::UMat2x3 => Uniform::UMat2x3(
				[[1, 0],
				 [0, 1],
				 [0, 0]]
			),
			UniformType::UMat2x4 => Uniform::UMat2x4(
				[[1, 0],
				 [0, 1],
				 [0, 0],
				 [0, 0]]
			),
			UniformType::UMat3 => Uniform::UMat3(
				[[1, 0, 0],
				 [0, 1, 0],
				 [0, 0, 1]]
			),
			UniformType::UMat3x2 => Uniform::UMat3x2(
				[[1, 0, 0],
				 [0 ,1, 0]]
			),
			UniformType::UMat3x4 => Uniform::UMat3x4(
				[[1, 0, 0],
				 [0, 1, 0],
				 [0, 0, 1],
				 [0, 0, 0]]
			),
			UniformType::UMat4 => Uniform::UMat4(
				[[1, 0, 0, 0],
				 [0, 1, 0, 0],
				 [0, 0, 1, 0],
				 [0, 0, 0, 1]]
			),
			UniformType::UMat4x2 => Uniform::UMat4x2(
				[[1, 0, 0, 0],
				 [0, 1, 0, 0]]
			),
			UniformType::UMat4x3 => Uniform::UMat4x3(
				[[1, 0, 0, 0],
				 [0, 1, 0, 0],
				 [0, 0, 1, 0]]
			),
		}
	}

	fn into_binary(uniforms: &[Uniform]) -> Vec<u8> {
		let mut binary = Vec::new();

		for uniform in uniforms {
			match uniform {
				Uniform::Float(data) => scalar_to_binary!(data, binary, to_le_bytes),
    			Uniform::Vec2(data) => vec_to_binary!(data, binary, to_le_bytes),
    			Uniform::Vec3(data) => vec_to_binary!(data, binary, to_le_bytes),
    			Uniform::Vec4(data) => vec_to_binary!(data, binary, to_le_bytes),
    			Uniform::Mat2(data) => mat_to_binary!(data, binary, to_le_bytes),
    			Uniform::Mat2x3(data) => mat_to_binary!(data, binary, to_le_bytes),
    			Uniform::Mat2x4(data) => mat_to_binary!(data, binary, to_le_bytes),
    			Uniform::Mat3(data) => mat_to_binary!(data, binary, to_le_bytes),
    			Uniform::Mat3x2(data) => mat_to_binary!(data, binary, to_le_bytes),
    			Uniform::Mat3x4(data) => mat_to_binary!(data, binary, to_le_bytes),
    			Uniform::Mat4(data) => mat_to_binary!(data, binary, to_le_bytes),
    			Uniform::Mat4x2(data) => mat_to_binary!(data, binary, to_le_bytes),
    			Uniform::Mat4x3(data) => mat_to_binary!(data, binary, to_le_bytes),

    			Uniform::Double(data) => scalar_to_binary!(data, binary, to_le_bytes),
				Uniform::DVec2(data) => vec_to_binary!(data, binary, to_le_bytes),
				Uniform::DVec3(data) => vec_to_binary!(data, binary, to_le_bytes),
				Uniform::DVec4(data) => vec_to_binary!(data, binary, to_le_bytes),
				Uniform::DMat2(data) => mat_to_binary!(data, binary, to_le_bytes),
				Uniform::DMat2x3(data) => mat_to_binary!(data, binary, to_le_bytes),
				Uniform::DMat2x4(data) => mat_to_binary!(data, binary, to_le_bytes),
				Uniform::DMat3(data) => mat_to_binary!(data, binary, to_le_bytes),
				Uniform::DMat3x2(data) => mat_to_binary!(data, binary, to_le_bytes),
				Uniform::DMat3x4(data) => mat_to_binary!(data, binary, to_le_bytes),
				Uniform::DMat4(data) => mat_to_binary!(data, binary, to_le_bytes),
				Uniform::DMat4x2(data) => mat_to_binary!(data, binary, to_le_bytes),
				Uniform::DMat4x3(data) => mat_to_binary!(data, binary, to_le_bytes),

				Uniform::Int(data) => scalar_to_binary!(data, binary, to_le_bytes),
				Uniform::IVec2(data) => vec_to_binary!(data, binary, to_le_bytes),
				Uniform::IVec3(data) => vec_to_binary!(data, binary, to_le_bytes),
				Uniform::IVec4(data) => vec_to_binary!(data, binary, to_le_bytes),
				Uniform::IMat2(data) => mat_to_binary!(data, binary, to_le_bytes),
				Uniform::IMat2x3(data) => mat_to_binary!(data, binary, to_le_bytes),
				Uniform::IMat2x4(data) => mat_to_binary!(data, binary, to_le_bytes),
				Uniform::IMat3(data) => mat_to_binary!(data, binary, to_le_bytes),
				Uniform::IMat3x2(data) => mat_to_binary!(data, binary, to_le_bytes),
				Uniform::IMat3x4(data) => mat_to_binary!(data, binary, to_le_bytes),
				Uniform::IMat4(data) => mat_to_binary!(data, binary, to_le_bytes),
				Uniform::IMat4x2(data) => mat_to_binary!(data, binary, to_le_bytes),
				Uniform::IMat4x3(data) => mat_to_binary!(data, binary, to_le_bytes),

				Uniform::UInt(data) => scalar_to_binary!(data, binary, to_le_bytes),
				Uniform::UVec2(data) => vec_to_binary!(data, binary, to_le_bytes),
				Uniform::UVec3(data) => vec_to_binary!(data, binary, to_le_bytes),
				Uniform::UVec4(data) => vec_to_binary!(data, binary, to_le_bytes),
				Uniform::UMat2(data) => mat_to_binary!(data, binary, to_le_bytes),
				Uniform::UMat2x3(data) => mat_to_binary!(data, binary, to_le_bytes),
				Uniform::UMat2x4(data) => mat_to_binary!(data, binary, to_le_bytes),
				Uniform::UMat3(data) => mat_to_binary!(data, binary, to_le_bytes),
				Uniform::UMat3x2(data) => mat_to_binary!(data, binary, to_le_bytes),
				Uniform::UMat3x4(data) => mat_to_binary!(data, binary, to_le_bytes),
				Uniform::UMat4(data) => mat_to_binary!(data, binary, to_le_bytes),
				Uniform::UMat4x2(data) => mat_to_binary!(data, binary, to_le_bytes),
				Uniform::UMat4x3(data) => mat_to_binary!(data, binary, to_le_bytes),
			}
		}

		return binary;
	}
}

pub(crate) struct RenderObjectInternal {
	surfaces: Vec<Uuid>,
	mesh_data: MeshDataInternal,
	graphics_program: GraphicsProgramInternal,

	descriptors: Vec<(Descriptor, DescriptorData)>,
}

impl RenderCall for RenderObjectInternal {
	fn render_call<'a>(&self, builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>) -> &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer> {
		builder
			.bind_vertex_buffers(0, self.mesh_data.vertices.clone()).unwrap()
			.bind_index_buffer(self.mesh_data.indices.clone()).unwrap()
			.bind_pipeline_graphics(self.graphics_program.pipeline.clone()).unwrap();
		
		let sets = self.descriptors.chunk_by(|(a, _), (b, _)| a.set == b.set);
		for set_descriptors in sets {
			let mut set = 0;
			let mut set_writes = Vec::with_capacity(set_descriptors.len());
			for (descriptor, data) in set_descriptors {
				set = descriptor.set;
				set_writes.push(data.to_descriptor_write(descriptor.binding));
			}
			
			builder.push_descriptor_set(
				vulkano::pipeline::PipelineBindPoint::Graphics, 
				self.graphics_program.pipeline.layout().clone(), 
				set, 
				set_writes.into()
			).unwrap();
		}

		unsafe { builder
			.draw_indexed(self.mesh_data.indices.len() as u32, 1, 0, 0, 0).unwrap()
		}
	}

	fn get_render_surface_uuids(&self) -> Vec<Uuid> {
		return self.surfaces.clone();
	}
}

impl RenderEngine {
	pub fn create_render_object(&mut self, mesh_data: MeshData, graphics_program: GraphicsProgram) -> Result<RenderObjectHandle, ()> {
		let uuid = Uuid::now_v7();

		let mesh_data = unwrap_option_or_none!(self.mesh_data.get(&mesh_data.uuid));
		let graphics_program = unwrap_option_or_none!(self.graphics_programs.get(&graphics_program.uuid));

		let mut descriptors = Vec::new();
		for descriptor in &graphics_program.descriptors {
			descriptors.push((descriptor.clone(), DescriptorData::from_descriptor(descriptor, self.buffer_allocator.clone())))
		}

		let internal = RenderObjectInternal {
			surfaces: Vec::new(),
			mesh_data: mesh_data.clone(),
			graphics_program: graphics_program.clone(),
			descriptors: descriptors
		};

		self.render_targets.insert(uuid, RenderTarget::Object(internal));

		Ok(RenderObjectHandle {
			uuid: uuid,
		})
	}

	pub fn render_render_object(&mut self, handle: RenderObjectHandle) -> Result<(), ()> {
		let render_caller = unwrap_option_or_none!(self.render_targets.get(&handle.uuid)).to_render_caller();
		let uuids = render_caller.get_render_surface_uuids();
		if uuids.len() == 0 {
			for (_, render_surface) in &mut self.render_surfaces {
				render_surface.process_render_queue(render_caller);
			}
		} else {
			for uuid in &uuids {
				unwrap_option_or_none!(self.render_surfaces.get_mut(uuid)).process_render_queue(render_caller);
			}
		}

		Ok(())
	}
}