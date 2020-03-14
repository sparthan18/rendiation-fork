use rendiation_math_entity::Line3;
use crate::vertex::Vertex;
use core::marker::PhantomData;
use rendiation_math_entity::Face;

pub trait PrimitiveFromGeometryData {
  fn from_data(index: &[u16], data: &[Vertex], offset: usize) -> Self;
}

impl PrimitiveFromGeometryData for Face {
  fn from_data(index: &[u16], data: &[Vertex], offset: usize) -> Self {
    let a = data[index[offset] as usize].position;
    let b = data[index[offset + 1] as usize].position;
    let c = data[index[offset + 2] as usize].position;
    Face { a, b, c }
  }
}

impl PrimitiveFromGeometryData for Line3 {
  fn from_data(index: &[u16], data: &[Vertex], offset: usize) -> Self {
    let start = data[index[offset] as usize].position;
    let end = data[index[offset + 1] as usize].position;
    Line3 { start, end }
  }
}

pub trait PrimitiveTopology {
  type Primitive: PrimitiveFromGeometryData;
  const STRIDE: usize;
  const WGPU_ENUM: wgpu::PrimitiveTopology;
}

pub struct TriangleList;

impl PrimitiveTopology for TriangleList {
  type Primitive = Face;
  const STRIDE: usize = 3;
  const WGPU_ENUM: wgpu::PrimitiveTopology = wgpu::PrimitiveTopology::TriangleList;
}

pub struct LineList;

impl PrimitiveTopology for LineList {
  type Primitive = Line3;
  const STRIDE: usize = 2;
  const WGPU_ENUM: wgpu::PrimitiveTopology = wgpu::PrimitiveTopology::LineList;
}

pub struct PrimitiveIter<'a, T: PrimitiveFromGeometryData> {
  pub index: &'a [u16],
  pub data: &'a [Vertex],
  pub current: usize,
  pub _phantom: PhantomData<T>,
}

impl<'a, T: PrimitiveFromGeometryData> Iterator for PrimitiveIter<'a, T> {
  type Item = T;

  fn next(&mut self) -> Option<T> {
    if self.current == self.index.len() {
      None
    } else {
      Some(T::from_data(self.index, self.data, self.current))
    }
  }
}
