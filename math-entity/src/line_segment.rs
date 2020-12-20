use rendiation_math::{Lerp, Scalar, SquareMatrixType, VectorImpl, VectorType};

use crate::{Positioned, SpaceEntity, SpaceLineSegment};

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct LineSegment<V> {
  pub start: V,
  pub end: V,
}

impl<T: Scalar, V: Positioned<T, D>, const D: usize> SpaceEntity<T, D> for LineSegment<V> {
  fn apply_matrix(&mut self, mat: SquareMatrixType<T, D>) -> &mut Self {
    self.start.position_mut().apply_matrix(mat);
    self.end.position_mut().apply_matrix(mat);
    self
  }
}

impl<T, V, const D: usize> SpaceLineSegment<T, D> for LineSegment<V>
where
  T: Scalar,
  V: Positioned<T, D>,
  VectorType<T, D>: VectorImpl,
{
  fn start(&self) -> VectorType<T, D> {
    self.start.position()
  }
  fn end(&self) -> VectorType<T, D> {
    self.end.position()
  }
  fn sample(&self, t: T) -> VectorType<T, D> {
    self.start().lerp(self.end(), t)
  }
}

impl<V> LineSegment<V> {
  pub fn new(start: V, end: V) -> Self {
    Self { start, end }
  }

  pub fn iter_point(&self) -> LineSegmentIter<'_, V> {
    LineSegmentIter::new(self)
  }
}

pub struct LineSegmentIter<'a, V> {
  line_segment: &'a LineSegment<V>,
  visit_count: i8,
}

impl<'a, V> LineSegmentIter<'a, V> {
  pub fn new(line3: &'a LineSegment<V>) -> Self {
    Self {
      line_segment: line3,
      visit_count: -1,
    }
  }
}

impl<'a, V: Copy> Iterator for LineSegmentIter<'a, V> {
  type Item = V;
  fn next(&mut self) -> Option<Self::Item> {
    self.visit_count += 1;
    if self.visit_count == 0 {
      Some(self.line_segment.start)
    } else if self.visit_count == 1 {
      Some(self.line_segment.end)
    } else {
      None
    }
  }
}

impl<V: Copy> LineSegment<V> {
  pub fn swap(&self) -> Self {
    Self::new(self.end, self.start)
  }

  pub fn swap_if(&self, prediction: impl FnOnce(Self) -> bool) -> Self {
    if prediction(*self) {
      self.swap()
    } else {
      *self
    }
  }
}
