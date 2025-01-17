use std::{
  ops::{Deref, DerefMut},
  sync::{RwLockReadGuard, RwLockWriteGuard, Weak},
};

use futures::{Future, Stream};
use reactive::{do_updates, ReactiveMapping};

use super::identity::Identity;
use crate::*;

#[derive(Default)]
pub struct SceneItemRef<T: IncrementalBase> {
  inner: Arc<RwLock<Identity<T>>>,

  // we keep this id on the self, to avoid unnecessary read lock.
  id: usize,
}

pub struct SceneItemWeakRef<T: IncrementalBase> {
  inner: Weak<RwLock<Identity<T>>>,
  id: usize,
}

impl<T: IncrementalBase> SceneItemWeakRef<T> {
  pub fn upgrade(&self) -> Option<SceneItemRef<T>> {
    self
      .inner
      .upgrade()
      .map(|inner| SceneItemRef { inner, id: self.id })
  }
}

impl<T: IncrementalBase + Send + Sync> IncrementalBase for SceneItemRef<T> {
  type Delta = Self;

  fn expand(&self, mut cb: impl FnMut(Self::Delta)) {
    cb(self.clone())
  }
}

impl<T: ApplicableIncremental + Send + Sync> ApplicableIncremental for SceneItemRef<T> {
  type Error = T::Error;

  fn apply(&mut self, delta: Self::Delta) -> Result<(), Self::Error> {
    *self = delta;
    Ok(())
  }
}

impl<T: IncrementalBase> Clone for SceneItemRef<T> {
  fn clone(&self) -> Self {
    Self {
      inner: self.inner.clone(),
      id: self.id,
    }
  }
}

impl<T: IncrementalBase> Hash for SceneItemRef<T> {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.id.hash(state);
  }
}

impl<T: IncrementalBase> PartialEq for SceneItemRef<T> {
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id
  }
}
impl<T: IncrementalBase> Eq for SceneItemRef<T> {}

impl<T: IncrementalBase> From<T> for SceneItemRef<T> {
  fn from(inner: T) -> Self {
    Self::new(inner)
  }
}

pub struct Mutating<'a, T: IncrementalBase> {
  pub(crate) inner: &'a mut T,
  pub(crate) collector: &'a mut dyn FnMut(&T::Delta),
}

impl<'a, T: IncrementalBase> Deref for Mutating<'a, T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    self.inner
  }
}

impl<'a, T: ApplicableIncremental> Mutating<'a, T> {
  pub fn modify(&mut self, delta: T::Delta) {
    if self.inner.should_apply_hint(&delta) {
      (self.collector)(&delta);
      self.inner.apply(delta).unwrap()
    }
  }
}

impl<'a, T: IncrementalBase> Mutating<'a, T> {
  /// # Safety
  /// the mutation should be record manually
  pub unsafe fn get_mut_ref(&mut self) -> &mut T {
    self.inner
  }

  /// # Safety
  /// the mutation will be not apply on original data
  pub unsafe fn trigger_change_but_not_apply(&mut self, delta: T::Delta) {
    (self.collector)(&delta);
  }
}

pub trait ModifySceneItemDelta<T: IncrementalBase> {
  fn apply_modify(self, target: &SceneItemRef<T>);
}

impl<T, X> ModifySceneItemDelta<T> for X
where
  T: ApplicableIncremental<Delta = X>,
{
  fn apply_modify(self, target: &SceneItemRef<T>) {
    target.mutate(|mut m| {
      m.modify(self);
    })
  }
}

impl<T: IncrementalBase> GlobalIdentified for SceneItemRef<T> {
  fn guid(&self) -> usize {
    self.id
  }
}
impl<T: IncrementalBase> AsRef<dyn GlobalIdentified> for SceneItemRef<T> {
  fn as_ref(&self) -> &(dyn GlobalIdentified + 'static) {
    self
  }
}
impl<T: IncrementalBase> AsMut<dyn GlobalIdentified> for SceneItemRef<T> {
  fn as_mut(&mut self) -> &mut (dyn GlobalIdentified + 'static) {
    self
  }
}

impl<T: IncrementalBase> SceneItemRef<T> {
  pub fn new(source: T) -> Self {
    let inner = Identity::new(source);
    let id = inner.id;
    let inner = Arc::new(RwLock::new(inner));
    Self { inner, id }
  }

  pub fn downgrade(&self) -> SceneItemWeakRef<T> {
    SceneItemWeakRef {
      inner: Arc::downgrade(&self.inner),
      id: self.id,
    }
  }

  pub fn defer_weak(&self) -> impl Fn(()) -> Option<Self> {
    let weak = self.downgrade();
    move |_| weak.upgrade()
  }

  pub fn pass_changes_to(
    &self,
    other: &Self,
    mut extra_mapper: impl FnMut(T::Delta) -> T::Delta + Send + Sync + 'static,
  ) where
    T: ApplicableIncremental,
  {
    let other_weak = other.downgrade();
    let remove_token = self.read().delta_source.on(move |delta| {
      if let Some(other) = other_weak.upgrade() {
        other.mutate(|mut m| m.modify(extra_mapper(delta.clone())));
        false
      } else {
        true
      }
    });

    let self_weak = self.downgrade();
    other.read().drop_source.on(move |_| {
      if let Some(origin) = self_weak.upgrade() {
        origin.read().delta_source.off(remove_token)
      }
    });
  }

  pub fn trigger_change(&self, delta: &T::Delta) {
    // ignore lock poison
    let inner = self.inner.read().unwrap_or_else(|e| e.into_inner());
    let data: &T = &inner;
    let view = &DeltaView { data, delta };
    let view = unsafe { std::mem::transmute(view) };
    inner.delta_source.emit(view);
  }

  pub fn mutate<R>(&self, mutator: impl FnOnce(Mutating<T>) -> R) -> R {
    // ignore lock poison
    let mut inner = self.inner.write().unwrap_or_else(|e| e.into_inner());
    let i: &mut Identity<T> = &mut inner;
    i.mutate(mutator)
  }
  pub fn visit<R>(&self, mut visitor: impl FnMut(&T) -> R) -> R {
    // ignore lock poison
    let inner = self.inner.read().unwrap_or_else(|e| e.into_inner());
    visitor(&inner)
  }

  pub fn read(&self) -> SceneItemRefGuard<T> {
    // ignore lock poison
    let inner = self.inner.read().unwrap_or_else(|e| e.into_inner());
    SceneItemRefGuard { inner }
  }

  pub fn write_unchecked(&self) -> SceneItemRefMutGuard<T> {
    // ignore lock poison
    let inner = self.inner.write().unwrap_or_else(|e| e.into_inner());
    SceneItemRefMutGuard { inner }
  }
}

pub struct SceneItemRefGuard<'a, T: IncrementalBase> {
  inner: RwLockReadGuard<'a, Identity<T>>,
}

impl<'a, T: IncrementalBase> Deref for SceneItemRefGuard<'a, T> {
  type Target = Identity<T>;

  fn deref(&self) -> &Self::Target {
    self.inner.deref()
  }
}

pub struct SceneItemRefMutGuard<'a, T: IncrementalBase> {
  inner: RwLockWriteGuard<'a, Identity<T>>,
}

impl<'a, T: IncrementalBase> Deref for SceneItemRefMutGuard<'a, T> {
  type Target = Identity<T>;

  fn deref(&self) -> &Self::Target {
    self.inner.deref()
  }
}

impl<'a, T: IncrementalBase> DerefMut for SceneItemRefMutGuard<'a, T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.inner.deref_mut()
  }
}

pub trait IntoSceneItemRef: Sized + IncrementalBase {
  fn into_ref(self) -> SceneItemRef<Self> {
    self.into()
  }
}

impl<T: IncrementalBase> IntoSceneItemRef for T {}

pub trait SceneItemReactiveMapping<M> {
  type ChangeStream: Stream + Unpin;
  type Ctx<'a>;

  fn build(&self, ctx: &Self::Ctx<'_>) -> (M, Self::ChangeStream);

  fn update(&self, mapped: &mut M, change: &mut Self::ChangeStream, ctx: &Self::Ctx<'_>);
}

impl<M, T> ReactiveMapping<M> for SceneItemRef<T>
where
  T: IncrementalBase + Send + Sync + 'static,
  Self: SceneItemReactiveMapping<M>,
{
  type ChangeStream = <Self as SceneItemReactiveMapping<M>>::ChangeStream;
  type DropFuture = impl Future<Output = ()> + Unpin;
  type Ctx<'a> = <Self as SceneItemReactiveMapping<M>>::Ctx<'a>;

  fn key(&self) -> usize {
    self.read().guid()
  }

  fn build(&self, ctx: &Self::Ctx<'_>) -> (M, Self::ChangeStream, Self::DropFuture) {
    let drop = self.create_drop();
    let (mapped, change) = SceneItemReactiveMapping::build(self, ctx);
    (mapped, change, drop)
  }

  fn update(&self, mapped: &mut M, change: &mut Self::ChangeStream, ctx: &Self::Ctx<'_>) {
    SceneItemReactiveMapping::update(self, mapped, change, ctx)
  }
}

pub trait SceneItemReactiveSimpleMapping<M> {
  type ChangeStream: Stream + Unpin;
  type Ctx<'a>;

  fn build(&self, ctx: &Self::Ctx<'_>) -> (M, Self::ChangeStream);
}

impl<M, T> SceneItemReactiveMapping<M> for SceneItemRef<T>
where
  T: IncrementalBase + Send + Sync + 'static,
  Self: SceneItemReactiveSimpleMapping<M>,
{
  type ChangeStream = <Self as SceneItemReactiveSimpleMapping<M>>::ChangeStream;
  type Ctx<'a> = <Self as SceneItemReactiveSimpleMapping<M>>::Ctx<'a>;

  fn build(&self, ctx: &Self::Ctx<'_>) -> (M, Self::ChangeStream) {
    SceneItemReactiveSimpleMapping::build(self, ctx)
  }

  fn update(&self, mapped: &mut M, change: &mut Self::ChangeStream, ctx: &Self::Ctx<'_>) {
    let mut pair = None;
    do_updates(change, |_| {
      pair = SceneItemReactiveMapping::build(self, ctx).into();
    });
    if let Some((new_mapped, new_change)) = pair {
      *mapped = new_mapped;
      *change = new_change;
    }
  }
}
