use rendiation_geometry::LineSegment;
use rendiation_geometry::Point;
use rendiation_geometry::Triangle;

use crate::*;

pub trait PrimitiveData<U> {
  fn from_data(data: &U, offset: usize) -> Self;
}

pub trait IndexedPrimitiveData<U, IU>: PrimitiveData<U> {
  type IndexIndicator;
  fn from_indexed_data(index: &IU, data: &U, offset: usize) -> Self;
  fn create_index_indicator(index: &IU, offset: usize) -> Self::IndexIndicator;
}

impl<T, U> PrimitiveData<U> for Triangle<T>
where
  T: Copy,
  U: IndexGet<Output = T>,
{
  #[inline(always)]
  fn from_data(data: &U, offset: usize) -> Self {
    let a = data.index_get(offset).unwrap();
    let b = data.index_get(offset + 1).unwrap();
    let c = data.index_get(offset + 2).unwrap();
    Triangle { a, b, c }
  }
}

impl<I, T, U, IU> IndexedPrimitiveData<U, IU> for Triangle<T>
where
  I: IndexType,
  T: Copy,
  U: IndexGet<Output = T>,
  IU: IndexGet<Output = I>,
{
  type IndexIndicator = Triangle<I>;
  #[inline(always)]
  fn from_indexed_data(index: &IU, data: &U, offset: usize) -> Self {
    let a = data
      .index_get(index.index_get(offset).unwrap().into_usize())
      .unwrap();
    let b = data
      .index_get(index.index_get(offset + 1).unwrap().into_usize())
      .unwrap();
    let c = data
      .index_get(index.index_get(offset + 2).unwrap().into_usize())
      .unwrap();
    Triangle { a, b, c }
  }

  #[inline(always)]
  fn create_index_indicator(index: &IU, offset: usize) -> Self::IndexIndicator {
    let a = index.index_get(offset).unwrap();
    let b = index.index_get(offset + 1).unwrap();
    let c = index.index_get(offset + 2).unwrap();
    Triangle { a, b, c }
  }
}

impl<T, U> PrimitiveData<U> for LineSegment<T>
where
  T: Copy,
  U: IndexGet<Output = T>,
{
  #[inline(always)]
  fn from_data(data: &U, offset: usize) -> Self {
    let start = data.index_get(offset).unwrap();
    let end = data.index_get(offset + 1).unwrap();
    LineSegment::line_segment(start, end)
  }
}

impl<I, T, U, IU> IndexedPrimitiveData<U, IU> for LineSegment<T>
where
  I: IndexType,
  T: Copy,
  U: IndexGet<Output = T>,
  IU: IndexGet<Output = I>,
{
  type IndexIndicator = LineSegment<I>;
  #[inline(always)]
  fn from_indexed_data(index: &IU, data: &U, offset: usize) -> Self {
    let start = data
      .index_get(index.index_get(offset).unwrap().into_usize())
      .unwrap();
    let end = data
      .index_get(index.index_get(offset + 1).unwrap().into_usize())
      .unwrap();
    LineSegment::line_segment(start, end)
  }
  #[inline(always)]
  fn create_index_indicator(index: &IU, offset: usize) -> Self::IndexIndicator {
    let start = index.index_get(offset).unwrap();
    let end = index.index_get(offset + 1).unwrap();
    LineSegment::line_segment(start, end)
  }
}

impl<T, U> PrimitiveData<U> for Point<T>
where
  T: Copy,
  U: IndexGet<Output = T>,
{
  #[inline(always)]
  fn from_data(data: &U, offset: usize) -> Self {
    Point(data.index_get(offset).unwrap())
  }
}

impl<I, T, U, IU> IndexedPrimitiveData<U, IU> for Point<T>
where
  I: IndexType,
  T: Copy,
  U: IndexGet<Output = T>,
  IU: IndexGet<Output = I>,
{
  type IndexIndicator = I;
  #[inline(always)]
  fn from_indexed_data(index: &IU, data: &U, offset: usize) -> Self {
    Point(
      data
        .index_get(index.index_get(offset).unwrap().into_usize())
        .unwrap(),
    )
  }

  #[inline(always)]
  fn create_index_indicator(index: &IU, offset: usize) -> Self::IndexIndicator {
    index.index_get(offset).unwrap()
  }
}

pub trait PrimitiveTopologyMeta: 'static {
  type Primitive<V>;
  const STEP: usize;
  const STRIDE: usize;
  const ENUM: PrimitiveTopology;
}

/// Primitive type the input mesh is composed of.
#[repr(C)]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum PrimitiveTopology {
  /// Vertex data is a list of points. Each vertex is a new point.
  PointList = 0,
  /// Vertex data is a list of lines. Each pair of vertices composes a new line.
  ///
  /// Vertices `0 1 2 3` create two lines `0 1` and `2 3`
  LineList = 1,
  /// Vertex data is a strip of lines. Each set of two adjacent vertices form a line.
  ///
  /// Vertices `0 1 2 3` create three lines `0 1`, `1 2`, and `2 3`.
  LineStrip = 2,
  /// Vertex data is a list of triangles. Each set of 3 vertices composes a new triangle.
  ///
  /// Vertices `0 1 2 3 4 5` create two triangles `0 1 2` and `3 4 5`
  TriangleList = 3,
  /// Vertex data is a triangle strip. Each set of three adjacent vertices form a triangle.
  ///
  /// Vertices `0 1 2 3 4 5` creates four triangles `0 1 2`, `2 1 3`, `3 2 4`, and `4 3 5`
  TriangleStrip = 4,
}

pub struct PointList;
impl PrimitiveTopologyMeta for PointList {
  type Primitive<T> = Point<T>;
  const STEP: usize = 1;
  const STRIDE: usize = 1;
  const ENUM: PrimitiveTopology = PrimitiveTopology::PointList;
}

pub struct TriangleList;
impl PrimitiveTopologyMeta for TriangleList {
  type Primitive<T> = Triangle<T>;
  const STEP: usize = 3;
  const STRIDE: usize = 3;
  const ENUM: PrimitiveTopology = PrimitiveTopology::TriangleList;
}

pub struct TriangleStrip;
impl PrimitiveTopologyMeta for TriangleStrip {
  type Primitive<T> = Triangle<T>;
  const STEP: usize = 1;
  const STRIDE: usize = 3;
  const ENUM: PrimitiveTopology = PrimitiveTopology::TriangleStrip;
}

pub struct LineList;
impl PrimitiveTopologyMeta for LineList {
  type Primitive<T> = LineSegment<T>;
  const STEP: usize = 2;
  const STRIDE: usize = 2;
  const ENUM: PrimitiveTopology = PrimitiveTopology::LineList;
}

pub struct LineStrip;
impl PrimitiveTopologyMeta for LineStrip {
  type Primitive<T> = LineSegment<T>;
  const STEP: usize = 1;
  const STRIDE: usize = 2;
  const ENUM: PrimitiveTopology = PrimitiveTopology::LineStrip;
}
