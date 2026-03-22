use std::{
	collections::HashMap, 
	sync::Arc
};

use uuid::Uuid;

use crate::{
	mesh_data::{
		MeshData, 
		mesh_data_internal::MeshDataInternal
	}, 
	pipeline::{
		Pipeline, 
		pipeline_internal::PipelineInternal
	}, 
	render_engine::render_thread::RenderThread
};

#[derive(Debug)]
pub(crate) struct RenderResources<'a> {
	mesh_data: &'a HashMap<Uuid, MeshDataInternal>,
	pipelines: &'a HashMap<Uuid, PipelineInternal>,
}

impl RenderThread {
	pub(crate) fn generate_render_resources(&self) -> RenderResources<'_> {
		RenderResources { 
			mesh_data: &self.mesh_data, 
			pipelines: &self.pipelines, 
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
}
