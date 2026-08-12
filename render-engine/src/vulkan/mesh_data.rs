
use std::sync::Arc;

use uuid::Uuid;
use vulkano::{
	buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer}, 
	command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferInfo, PrimaryAutoCommandBuffer}, 
	memory::allocator::{AllocationCreateInfo, MemoryTypeFilter}, 
	sync::{self, GpuFuture},
};

use crate::{
	data_formats::Vertex3D, 
	engine_command::MeshDataCommand, 
	vulkan::render_thread::{Operation, RenderThread},
};

pub struct MeshData {
	vertices: 	Subbuffer<[Vertex3D]>,
	indices:	Subbuffer<[u32]>,

	transfer_complete: Option<Operation>
}

impl MeshData {
	#[expect(unused)]
	pub fn bind<'a>(&mut self, builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>) -> Result<&'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, ()> {
		if self.transfer_complete.is_some() {
			let mut future = self.transfer_complete.take().unwrap();
			future.wait();
			future.cleanup_finished();
		}

		builder
			.bind_vertex_buffers(0, self.vertices.clone()).map_err(|_| ())?
			.bind_index_buffer(self.indices.clone()).map_err(|_| ())?;

		Ok(builder)
	}
}

impl RenderThread {
	pub fn process_mesh_data_command(&mut self, command: Box<MeshDataCommand>) {
		match *command {
			MeshDataCommand::CreateMeshData { vertices, indices, response } => { let _ = response.send(self.create_mesh_data(vertices, indices)); },
			MeshDataCommand::DropMeshData { uuid } => self.drop_mesh_data(uuid),
		}
	}

	fn create_mesh_data(&mut self, vertices: Box<[Vertex3D]>, indices: Box<[u32]>) -> Result<(Uuid,), ()> {
		let uuid = Uuid::now_v7();

		let mut builder = AutoCommandBufferBuilder::primary(
			self.command_allocator.clone(), 
			self.transfer_queue.queue_family_index(), 
			CommandBufferUsage::OneTimeSubmit,
		).map_err(|_| ())?;

		let vertex_final = Buffer::new_slice(
			self.buffer_allocator.clone(), 
			BufferCreateInfo {
				usage: BufferUsage::TRANSFER_DST | BufferUsage::VERTEX_BUFFER,
				..Default::default()
			}, 
			AllocationCreateInfo {
				memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
				..Default::default()
			},
			(vertices.len() * size_of::<Vertex3D>()) as u64,
		).map_err(|_| ())?;

		let vertex_initial = Buffer::from_iter(
			self.buffer_allocator.clone(), 
			BufferCreateInfo {
				usage: BufferUsage::TRANSFER_SRC,
				..Default::default()
			}, 
			AllocationCreateInfo {
				memory_type_filter: MemoryTypeFilter::HOST_SEQUENTIAL_WRITE | MemoryTypeFilter::PREFER_HOST,
				..Default::default()
			}, 
			vertices
		).map_err(|_| ())?;

		builder.copy_buffer(CopyBufferInfo::buffers(vertex_initial, vertex_final.clone())).map_err(|_| ())?;

		let index_final = Buffer::new_slice(
			self.buffer_allocator.clone(), 
			BufferCreateInfo {
				usage: BufferUsage::TRANSFER_DST | BufferUsage::INDEX_BUFFER,
				..Default::default()
			}, 
			AllocationCreateInfo {
				memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
				..Default::default()
			}, 
		(indices.len() * size_of::<u32>()) as u64,
		).map_err(|_| ())?;

		let index_initial = Buffer::from_iter(
			self.buffer_allocator.clone(), 
			BufferCreateInfo {
				usage: BufferUsage::TRANSFER_SRC,
				..Default::default()
			}, 
			AllocationCreateInfo {
				memory_type_filter: MemoryTypeFilter::HOST_SEQUENTIAL_WRITE | MemoryTypeFilter::PREFER_HOST,
				..Default::default()
			}, 
			indices
		).map_err(|_| ())?;

		builder.copy_buffer(CopyBufferInfo::buffers(index_initial, index_final.clone())).map_err(|_| ())?;

		let command_buffer = builder.build().map_err(|_| ())?;

		let future;
		if self.transfer_operation.future.is_some() {
			future = Arc::new(self.transfer_operation.future.take().unwrap()
				.then_execute(self.transfer_queue.clone(), command_buffer).map_err(|_| ())?.boxed_send()
				.then_signal_fence_and_flush().map_err(|_| ())?)
		} else {
			future = Arc::new(sync::now(self.device.clone())
				.then_execute(self.transfer_queue.clone(), command_buffer).map_err(|_| ())?.boxed_send()
				.then_signal_fence_and_flush().map_err(|_| ())?)
		}
		let operation = Operation::transfer(future);
		self.transfer_operation = operation.clone();

		self.mesh_data.insert(uuid, MeshData {
			vertices: 			vertex_final,
			indices: 			index_final,
			transfer_complete: 	Some(operation),
		});

		Ok((uuid,))
	}

	fn drop_mesh_data(&mut self, uuid: Uuid) {
		self.mesh_data.remove(&uuid);
	}
}
