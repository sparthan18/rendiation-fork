use std::marker::PhantomData;

use crate::{
  geometry::IndexPrimitiveTopologyMeta, geometry::IndexedPrimitiveData,
  geometry::PrimitiveTopologyMeta, geometry::TriangleList, vertex::Vertex,
};
use rendiation_geometry::Positioned;

use super::{AnyGeometry, AnyIndexGeometry, GeometryDataContainer};

pub struct IndexedGeometryView<'a, I, V = Vertex, T = TriangleList, U = Vec<V>> {
  pub data: &'a U,
  pub index: &'a Vec<I>,
  _v_phantom: PhantomData<V>,
  _phantom: PhantomData<T>,
}

#[allow(clippy::ptr_arg)]
impl<'a, I, V, T, U> IndexedGeometryView<'a, I, V, T, U> {
  pub fn new(v: &'a U, index: &'a Vec<I>) -> Self {
    Self {
      data: v,
      index,
      _v_phantom: PhantomData,
      _phantom: PhantomData,
    }
  }
}

impl<'a, I, V, T, U> AnyGeometry for IndexedGeometryView<'a, I, V, T, U>
where
  V: Positioned<f32, 3>,
  T: IndexPrimitiveTopologyMeta<I, V>,
  <T as PrimitiveTopologyMeta<V>>::Primitive: IndexedPrimitiveData<I, V, U, Vec<I>>,
  U: GeometryDataContainer<V>,
{
  type Primitive = T::Primitive;

  #[inline(always)]
  fn draw_count(&self) -> usize {
    self.index.len()
  }

  #[inline(always)]
  fn primitive_count(&self) -> usize {
    (self.index.len() - T::STRIDE) / T::STEP + 1
  }

  #[inline(always)]
  fn primitive_at(&self, primitive_index: usize) -> Self::Primitive {
    let index = primitive_index * T::STEP;
    T::Primitive::from_indexed_data(&self.index, &self.data, index)
  }
}

impl<'a, I, V, T, U> AnyIndexGeometry for IndexedGeometryView<'a, I, V, T, U>
where
  V: Positioned<f32, 3>,
  T: IndexPrimitiveTopologyMeta<I, V>,
  T::Primitive: IndexedPrimitiveData<I, V, U, Vec<I>>,
  U: GeometryDataContainer<V>,
{
  type IndexPrimitive = <T::Primitive as IndexedPrimitiveData<I, V, U, Vec<I>>>::IndexIndicator;

  fn index_primitive_at(&self, primitive_index: usize) -> Self::IndexPrimitive {
    let index = primitive_index * T::STEP;
    T::Primitive::create_index_indicator(&self.index, index)
  }
}