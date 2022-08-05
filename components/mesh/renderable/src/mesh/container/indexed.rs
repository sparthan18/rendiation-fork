use super::{
  super::{PrimitiveTopologyMeta, TriangleList},
  AbstractIndexMesh, AbstractMesh,
};
use crate::{mesh::IndexedPrimitiveData, vertex::Vertex};
use core::marker::PhantomData;
use std::hash::Hash;

/// We don't use TryInto<usize, Error: Debug> to express
/// the conversion between the usize and self, because we assume the range of IndexType not
/// exceeds usize. So their conversion is infallible. But the std not impl direct From trait
/// for u32/u16. To keep simplicity, we provide explicit trait function here
///
/// The reason we don't impl from_usize is this should impl by the index container
pub trait IndexType: Copy + Eq + Ord + Hash {
  fn into_usize(self) -> usize;
}
impl IndexType for u32 {
  fn into_usize(self) -> usize {
    self as usize
  }
}
impl IndexType for u16 {
  fn into_usize(self) -> usize {
    self as usize
  }
}

pub enum DynIndexContainer {
  Uint16(Vec<u16>),
  Uint32(Vec<u32>),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DynIndex {
  Uint16(u16),
  Uint32(u32),
}

impl IndexType for DynIndex {
  fn into_usize(self) -> usize {
    match self {
      DynIndex::Uint16(i) => i as usize,
      DynIndex::Uint32(i) => i as usize,
    }
  }
}

/// Mark type that indicates index oversized u32 and cannot used in gpu.
pub struct IndexOversized;

impl DynIndexContainer {
  pub fn is_u32_buffer(&self) -> bool {
    match self {
      DynIndexContainer::Uint16(_) => false,
      DynIndexContainer::Uint32(_) => true,
    }
  }

  pub fn push_index(&mut self, index: usize) -> Result<(), IndexOversized> {
    if index > u32::MAX as usize {
      Err(IndexOversized)
    } else {
      match self {
        DynIndexContainer::Uint16(buffer) => {
          if index > u16::MAX as usize {
            let buffer = self.try_upgrade_to_u32();
            buffer.push(index as u32)
          } else {
            buffer.push(index as u16)
          }
        }
        DynIndexContainer::Uint32(buffer) => buffer.push(index as u32),
      }
      Ok(())
    }
  }

  pub fn try_upgrade_to_u32(&mut self) -> &mut Vec<u32> {
    match self {
      DynIndexContainer::Uint16(buffer) => {
        *self = DynIndexContainer::Uint32(buffer.iter().map(|&i| i as u32).collect());
        self.try_upgrade_to_u32()
      }
      DynIndexContainer::Uint32(buffer) => buffer,
    }
  }
}

/// A indexed mesh that use vertex as primitive;
pub struct IndexedMesh<I = u16, V = Vertex, T = TriangleList, U = Vec<V>, IU = Vec<I>> {
  pub data: U,
  pub index: IU,
  _i_phantom: PhantomData<I>,
  _v_phantom: PhantomData<V>,
  _phantom: PhantomData<T>,
}

impl<I, V, T, U> From<(U, Vec<I>)> for IndexedMesh<I, V, T, U> {
  fn from(item: (U, Vec<I>)) -> Self {
    IndexedMesh::new(item.0, item.1)
  }
}

impl<I, V, T, U, IU> IndexedMesh<I, V, T, U, IU> {
  pub fn new(v: U, index: IU) -> Self {
    Self {
      data: v,
      index,
      _i_phantom: PhantomData,
      _v_phantom: PhantomData,
      _phantom: PhantomData,
    }
  }
}

impl<I, V, T, U, IU> AbstractMesh for IndexedMesh<I, V, T, U, IU>
where
  V: Copy,
  U: AsRef<[V]>,
  IU: AsRef<[I]>,
  T: PrimitiveTopologyMeta<V>,
  <T as PrimitiveTopologyMeta<V>>::Primitive: IndexedPrimitiveData<I, V, U, IU>,
{
  type Primitive = T::Primitive;

  #[inline(always)]
  fn draw_count(&self) -> usize {
    self.index.as_ref().len()
  }

  #[inline(always)]
  fn primitive_count(&self) -> usize {
    (self.index.as_ref().len() - T::STRIDE) / T::STEP + 1
  }

  #[inline(always)]
  fn primitive_at(&self, primitive_index: usize) -> Self::Primitive {
    let index = primitive_index * T::STEP;
    T::Primitive::from_indexed_data(&self.index, &self.data, index)
  }
}

impl<I, V, T, U, IU> AbstractIndexMesh for IndexedMesh<I, V, T, U, IU>
where
  V: Copy,
  U: AsRef<[V]>,
  IU: AsRef<[I]>,
  T: PrimitiveTopologyMeta<V>,
  T::Primitive: IndexedPrimitiveData<I, V, U, IU>,
{
  type IndexPrimitive = <T::Primitive as IndexedPrimitiveData<I, V, U, IU>>::IndexIndicator;

  #[inline(always)]
  fn index_primitive_at(&self, primitive_index: usize) -> Self::IndexPrimitive {
    let index = primitive_index * T::STEP;
    T::Primitive::create_index_indicator(&self.index, index)
  }
}
