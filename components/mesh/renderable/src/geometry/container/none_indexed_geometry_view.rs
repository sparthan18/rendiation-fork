use std::marker::PhantomData;

use rendiation_geometry::Positioned;

use crate::{
  geometry::PrimitiveData, geometry::PrimitiveTopologyMeta, geometry::TriangleList, vertex::Vertex,
};

use super::{AnyGeometry, GeometryDataContainer};

pub struct NoneIndexedGeometryView<'a, V = Vertex, T = TriangleList, U = Vec<V>> {
  pub data: &'a U,
  _v_phantom: PhantomData<V>,
  _phantom: PhantomData<T>,
}

impl<'a, V, T, U> NoneIndexedGeometryView<'a, V, T, U> {
  pub fn new(v: &'a U) -> Self {
    Self {
      data: v,
      _v_phantom: PhantomData,
      _phantom: PhantomData,
    }
  }
}

impl<'a, V, T, U> AnyGeometry for NoneIndexedGeometryView<'a, V, T, U>
where
  V: Positioned<f32, 3>,
  T: PrimitiveTopologyMeta<V>,
  U: GeometryDataContainer<V>,
  T::Primitive: PrimitiveData<V, U>,
{
  type Primitive = T::Primitive;

  #[inline(always)]
  fn draw_count(&self) -> usize {
    self.data.as_ref().len()
  }

  #[inline(always)]
  fn primitive_count(&self) -> usize {
    (self.data.as_ref().len() - T::STRIDE) / T::STEP + 1
  }

  #[inline(always)]
  fn primitive_at(&self, primitive_index: usize) -> Self::Primitive {
    let index = primitive_index * T::STEP;
    T::Primitive::from_data(&self.data, index)
  }
}