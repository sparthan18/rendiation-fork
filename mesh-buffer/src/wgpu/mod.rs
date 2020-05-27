use crate::geometry::primitive::PrimitiveTopology;
use crate::{
  geometry::{indexed_geometry::IndexedGeometry, LineList, TriangleList},
  vertex::Vertex,
};
use rendiation::*;
use std::ops::Range;

use lazy_static::lazy_static;
lazy_static! {
  static ref VERTEX_BUFFERS: Vec<VertexBufferDescriptor<'static>> =
    { vec![Vertex::get_buffer_layout_descriptor()] };
}

impl<'a, T: PrimitiveTopology + WGPUPrimitiveTopology> GeometryProvider for IndexedGeometry<T> {
  fn get_geometry_vertex_state_descriptor() -> wgpu::VertexStateDescriptor<'static> {
    wgpu::VertexStateDescriptor {
      index_format: wgpu::IndexFormat::Uint16,
      vertex_buffers: &VERTEX_BUFFERS,
    }
  }

  fn get_primitive_topology() -> wgpu::PrimitiveTopology {
    T::WGPU_ENUM
  }
}

pub trait WGPUPrimitiveTopology {
  const WGPU_ENUM: wgpu::PrimitiveTopology;
}
impl WGPUPrimitiveTopology for TriangleList {
  const WGPU_ENUM: wgpu::PrimitiveTopology = wgpu::PrimitiveTopology::TriangleList;
}
impl WGPUPrimitiveTopology for LineList {
  const WGPU_ENUM: wgpu::PrimitiveTopology = wgpu::PrimitiveTopology::LineList;
}

pub fn as_bytes<T>(vec: &[T]) -> &[u8] {
  unsafe {
    std::slice::from_raw_parts(
      (vec as *const [T]) as *const u8,
      ::std::mem::size_of::<T>() * vec.len(),
    )
  }
}

impl VertexProvider for Vertex {
  fn get_buffer_layout_descriptor() -> wgpu::VertexBufferDescriptor<'static> {
    use std::mem;
    wgpu::VertexBufferDescriptor {
      stride: mem::size_of::<Self>() as wgpu::BufferAddress,
      step_mode: wgpu::InputStepMode::Vertex,
      attributes: &[
        wgpu::VertexAttributeDescriptor {
          format: wgpu::VertexFormat::Float3,
          offset: 0,
          shader_location: 0,
        },
        wgpu::VertexAttributeDescriptor {
          format: wgpu::VertexFormat::Float3,
          offset: 4 * 3,
          shader_location: 1,
        },
        wgpu::VertexAttributeDescriptor {
          format: wgpu::VertexFormat::Float2,
          offset: 4 * 3 + 4 * 3,
          shader_location: 2,
        },
      ],
    }
  }
}

pub struct GPUGeometry<T: PrimitiveTopology = TriangleList> {
  geometry: IndexedGeometry<T>,
  data_changed: bool,
  index_changed: bool,
  gpu_data: Option<[WGPUBuffer; 1]>,
  gpu_index: Option<WGPUBuffer>,
}

impl<T: PrimitiveTopology> From<IndexedGeometry<T>> for GPUGeometry<T> {
  fn from(geometry: IndexedGeometry<T>) -> Self {
    GPUGeometry {
      geometry,
      data_changed: true,
      index_changed: true,
      gpu_data: None,
      gpu_index: None,
    }
  }
}

impl From<(Vec<Vertex>, Vec<u16>)> for GPUGeometry {
  fn from(item: (Vec<Vertex>, Vec<u16>)) -> Self {
    IndexedGeometry::new(item.0, item.1).into()
  }
}

impl<T: PrimitiveTopology> GPUGeometry<T> {
  pub fn mutate_data(&mut self) -> &mut Vec<Vertex> {
    self.data_changed = true;
    &mut self.geometry.data
  }

  pub fn mutate_index(&mut self) -> &mut Vec<u16> {
    self.index_changed = true;
    &mut self.geometry.index
  }

  pub fn update_gpu(&mut self, renderer: &mut WGPURenderer) {
    if let Some(gpu_data) = &mut self.gpu_data {
      if self.data_changed {
        gpu_data[0].update(renderer, as_bytes(&self.geometry.data));
      }
    } else {
      self.gpu_data = Some([WGPUBuffer::new(
        renderer,
        as_bytes(&self.geometry.data),
        wgpu::BufferUsage::VERTEX,
      )])
    }

    if let Some(gpu_index) = &mut self.gpu_index {
      if self.index_changed {
        gpu_index.update(renderer, as_bytes(&self.geometry.index));
      }
    } else {
      self.gpu_index = Some(WGPUBuffer::new(
        renderer,
        as_bytes(&self.geometry.index),
        wgpu::BufferUsage::INDEX,
      ))
    }
  }

  pub fn get_vertex_buffer_unwrap(&self) -> &WGPUBuffer {
    if let Some(gpu_data) = &self.gpu_data {
      &gpu_data[0]
    } else {
      panic!("geometry not prepared")
    }
  }

  pub fn get_index_buffer_unwrap(&self) -> &WGPUBuffer {
    if let Some(gpu_index) = &self.gpu_index {
      &gpu_index
    } else {
      panic!("geometry not prepared")
    }
  }

  pub fn get_draw_range(&self) -> Range<u32> {
    0..self.geometry.get_full_count()
  }

  pub fn provide_geometry<'a, 'b: 'a>(&'b self, pass: &mut WGPURenderPass<'a>) {
    pass
      .gpu_pass
      .set_vertex_buffer(0, self.get_vertex_buffer_unwrap().get_gpu_buffer(), 0, 0);

    pass
      .gpu_pass
      .set_index_buffer(self.get_index_buffer_unwrap().get_gpu_buffer(), 0, 0);
  }

  pub fn render<'a, 'b: 'a>(&'b self, pass: &mut WGPURenderPass<'a>) {
    self.provide_geometry(pass);
    pass
      .gpu_pass
      .draw_indexed(0..self.geometry.get_full_count(), 0, 0..1);
  }
}