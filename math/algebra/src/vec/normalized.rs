use std::{marker::PhantomData, ops::*};

use crate::*;

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct NormalizedVector<T, V> {
  pub value: V,
  phantom: PhantomData<T>,
}

pub trait IntoNormalizedVector<T, V> {
  fn into_normalized(&self) -> NormalizedVector<T, V>;
  unsafe fn into_normalized_unchecked(&self) -> NormalizedVector<T, V>;
}

impl<T: Scalar, V: InnerProductSpace<T>> IntoNormalizedVector<T, V> for V {
  #[inline(always)]
  fn into_normalized(&self) -> NormalizedVector<T, V> {
    unsafe { NormalizedVector::wrap(self.normalize()) }
  }
  #[inline(always)]
  unsafe fn into_normalized_unchecked(&self) -> NormalizedVector<T, V> {
    NormalizedVector::wrap(*self)
  }
}

impl<T, V> NormalizedVector<T, V> {
  #[inline(always)]
  pub unsafe fn wrap(v: V) -> Self {
    Self {
      value: v,
      phantom: PhantomData,
    }
  }
}

impl<T: Scalar, V: InnerProductSpace<T>> NormalizedVector<T, V> {
  #[inline]
  pub fn normalize(&self) -> Self {
    *self
  }

  /// self should be normalized
  ///
  /// of course input normal should also be normalized
  #[inline]
  pub fn reflect(&self, normal: Self) -> Self {
    unsafe { self.value.reflect(*normal).into_normalized_unchecked() }
  }

  #[inline]
  pub fn length(&self) -> T {
    T::one()
  }

  #[inline]
  pub fn length2(&self) -> T {
    T::one()
  }

  #[inline]
  pub fn reverse(&self) -> Self {
    unsafe { self.value.reverse().into_normalized_unchecked() }
  }
}

pub trait InnerData<T> {
  fn get_inner(self) -> T;
}

impl<T, V> InnerData<V> for NormalizedVector<T, V> {
  #[inline(always)]
  fn get_inner(self) -> V {
    self.value
  }
}
impl<V> InnerData<V> for V {
  #[inline(always)]
  fn get_inner(self) -> V {
    self
  }
}

// after add / sub / mul scalar, the vector may not be normalized
impl<T, V: VectorSpace<T>, Rhs: InnerData<V>> Add<Rhs> for NormalizedVector<T, V> {
  type Output = V;
  #[inline(always)]
  fn add(self, rhs: Rhs) -> Self::Output {
    self.value + rhs.get_inner()
  }
}
impl<T, V: VectorSpace<T>, Rhs: InnerData<V>> Sub<Rhs> for NormalizedVector<T, V> {
  type Output = V;
  #[inline(always)]
  fn sub(self, rhs: Rhs) -> Self::Output {
    self.value - rhs.get_inner()
  }
}
impl<T, V: VectorSpace<T>> Mul<T> for NormalizedVector<T, V> {
  type Output = V;
  #[inline(always)]
  fn mul(self, rhs: T) -> Self::Output {
    self.value * rhs
  }
}
impl<T, V: VectorSpace<T>> Div<T> for NormalizedVector<T, V> {
  type Output = V;
  #[inline(always)]
  fn div(self, rhs: T) -> Self::Output {
    self.value / rhs
  }
}

impl<T, V> Deref for NormalizedVector<T, V> {
  type Target = V;
  #[inline(always)]
  fn deref(&self) -> &Self::Target {
    &self.value
  }
}
impl<T, V> DerefMut for NormalizedVector<T, V> {
  #[inline(always)]
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.value
  }
}

#[test]
fn test() {
  use crate::*;
  let a = Vec3::new(1., 1., 1.).into_normalized();
  let a = a.normalize(); // should use cheaper method
  let b = Vec3::new(1., 1., 1.);
  let _c = *a + b;
  let _c = a + a;
  let _c = a + b;
  let _nc = _c.normalize(); // ra code jump is misleading, but it actually used correct impl
}