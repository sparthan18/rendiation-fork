use crate::*;
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

#[derive(Debug, Clone)]
pub enum DynIndexContainer {
  Uint16(Vec<u16>),
  Uint32(Vec<u32>),
}

impl Default for DynIndexContainer {
  fn default() -> Self {
    Self::Uint16(Default::default())
  }
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
#[derive(Debug)]
pub struct IndexOversized;

impl DynIndexContainer {
  pub fn is_u32_buffer(&self) -> bool {
    match self {
      DynIndexContainer::Uint16(_) => false,
      DynIndexContainer::Uint32(_) => true,
    }
  }

  pub fn try_push_index(&mut self, index: usize) -> Result<(), IndexOversized> {
    if index > u32::MAX as usize {
      Err(IndexOversized)
    } else {
      self.push_index(index as u32);
      Ok(())
    }
  }

  pub fn push_index_clamped_u32(&mut self, index: usize) {
    let index = u32::MAX.min(index as u32);
    self.push_index(index)
  }

  fn push_index(&mut self, index: u32) {
    match self {
      DynIndexContainer::Uint16(buffer) => {
        if index > u16::MAX as u32 {
          let buffer = self.check_upgrade_to_u32();
          buffer.push(index as u32)
        } else {
          buffer.push(index as u16)
        }
      }
      DynIndexContainer::Uint32(buffer) => buffer.push(index as u32),
    }
  }

  pub fn check_upgrade_to_u32(&mut self) -> &mut Vec<u32> {
    match self {
      DynIndexContainer::Uint16(buffer) => {
        *self = DynIndexContainer::Uint32(buffer.iter().map(|&i| i as u32).collect());
        self.check_upgrade_to_u32()
      }
      DynIndexContainer::Uint32(buffer) => buffer,
    }
  }
}

impl FromIterator<usize> for DynIndexContainer {
  fn from_iter<T: IntoIterator<Item = usize>>(iter: T) -> Self {
    let mut c = Self::default();
    iter.into_iter().for_each(|i| c.try_push_index(i).unwrap());
    c
  }
}

pub struct DynIndexContainerIter<'a> {
  container: &'a DynIndexContainer,
  current: usize,
  count: usize,
}

impl<'a> Iterator for DynIndexContainerIter<'a> {
  type Item = DynIndex;

  fn next(&mut self) -> Option<Self::Item> {
    if self.current == self.count {
      None
    } else {
      let r = self.container.index_get(self.current).unwrap();
      self.current += 1;
      Some(r)
    }
  }
}

impl<'a> IntoIterator for &'a DynIndexContainer {
  type Item = DynIndex;

  type IntoIter = DynIndexContainerIter<'a>;

  fn into_iter(self) -> Self::IntoIter {
    DynIndexContainerIter {
      container: self,
      current: 0,
      count: match self {
        DynIndexContainer::Uint16(i) => i.len(),
        DynIndexContainer::Uint32(i) => i.len(),
      },
    }
  }
}

pub struct IndexBuffer<T> {
  inner: Vec<T>,
}

impl<I: TryFrom<usize>> TryFromIterator<usize> for IndexBuffer<I> {
  type Error = <I as std::convert::TryFrom<usize>>::Error;
  fn try_from_iter<T: IntoIterator<Item = usize>>(iter: T) -> Result<Self, Self::Error> {
    let inner: Result<Vec<I>, Self::Error> = iter.into_iter().map(|i| I::try_from(i)).collect();
    let inner = inner?;
    Ok(Self { inner })
  }
}

type CopiedIter<'a, T: Copy + 'static> = impl Iterator<Item = T> + 'a;
fn get_iter_copied<T: Copy>(v: &[T]) -> CopiedIter<T> {
  v.iter().copied()
}

impl<'a, T: Copy + 'static> IntoIterator for &'a IndexBuffer<T> {
  type Item = T;

  type IntoIter = CopiedIter<'a, T>;

  fn into_iter(self) -> Self::IntoIter {
    get_iter_copied(&self.inner)
  }
}

impl<T: Copy> IndexGet for IndexBuffer<T> {
  type Output = T;

  fn index_get(&self, key: usize) -> Option<Self::Output> {
    self.inner.get(key).copied()
  }
}

impl IndexGet for DynIndexContainer {
  type Output = DynIndex;

  fn index_get(&self, key: usize) -> Option<Self::Output> {
    match self {
      DynIndexContainer::Uint16(i) => DynIndex::Uint16(i.index_get(key).unwrap()),
      DynIndexContainer::Uint32(i) => DynIndex::Uint32(i.index_get(key).unwrap()),
    }
    .into()
  }
}

impl<T> CollectionSize for IndexBuffer<T> {
  fn len(&self) -> usize {
    self.inner.len()
  }
}

impl CollectionSize for DynIndexContainer {
  fn len(&self) -> usize {
    match self {
      DynIndexContainer::Uint16(i) => i.len(),
      DynIndexContainer::Uint32(i) => i.len(),
    }
  }
}

/// A indexed mesh that use vertex as primitive;
pub struct IndexedMesh<T, U, IU> {
  pub vertex: U,
  pub index: IU,
  _phantom: PhantomData<T>,
}

impl<T, U, IU> From<(U, IU)> for IndexedMesh<T, U, IU> {
  fn from(item: (U, IU)) -> Self {
    IndexedMesh::new(item.0, item.1)
  }
}

impl<T, U, IU> IndexedMesh<T, U, IU> {
  pub fn new(v: U, index: IU) -> Self {
    Self {
      vertex: v,
      index,
      _phantom: PhantomData,
    }
  }
}

impl<T, U, IU> AbstractMesh for IndexedMesh<T, U, IU>
where
  Self: AbstractIndexMesh,
  U: VertexContainer,
  T: PrimitiveTopologyMeta,
  IndexPrimitiveOf<Self>: Functor<Unwrapped: IndexType>,
{
  // sadly we can not directly write T::Primitive<U::Output>
  type Primitive = FunctorMapped<IndexPrimitiveOf<Self>, U::Output>;

  #[inline(always)]
  fn primitive_count(&self) -> usize {
    AbstractIndexMesh::primitive_count(self)
  }

  #[inline(always)]
  fn primitive_at(&self, primitive_index: usize) -> Self::Primitive {
    let index = self.index_primitive_at(primitive_index);
    index.f_map(|i| self.vertex.index_get(i.into_usize()).unwrap())
  }
}

type IndexPrimitiveOf<T> = <T as AbstractIndexMesh>::IndexPrimitive;

impl<T, U, IU> AbstractIndexMesh for IndexedMesh<T, U, IU>
where
  IU: IndexContainer,
  T: PrimitiveTopologyMeta,
  T::Primitive<IU::Output>: PrimitiveData<IU>,
{
  type IndexPrimitive = T::Primitive<IU::Output>;

  #[inline(always)]
  fn primitive_count(&self) -> usize {
    (self.index.len() - T::STRIDE) / T::STEP + 1
  }

  #[inline(always)]
  fn index_primitive_at(&self, primitive_index: usize) -> Self::IndexPrimitive {
    let index = primitive_index * T::STEP;
    T::Primitive::<IU::Output>::from_data(&self.index, index)
  }
}

impl<T, U, IU: CollectionSize> GPUConsumableMeshBuffer for IndexedMesh<T, U, IU> {
  #[inline(always)]
  fn draw_count(&self) -> usize {
    self.index.len()
  }
}
