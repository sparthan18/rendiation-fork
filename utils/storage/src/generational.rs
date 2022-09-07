use crate::*;

use arena::{Arena, Handle as ArenaHandle};

pub struct GenerationalVecStorage;

impl<T> StorageBehavior<T> for GenerationalVecStorage {
  type Container = Arena<T>;
  type Handle = ArenaHandle<T>;

  fn insert(c: &mut Self::Container, v: T) -> Handle<T, Self> {
    Handle::new(c.insert(v))
  }
  fn remove(c: &mut Self::Container, handle: Self::Handle) -> Option<T> {
    c.remove(handle)
  }
  fn get(c: &Self::Container, handle: Self::Handle) -> Option<&T> {
    c.get(handle)
  }
  fn get_mut(c: &mut Self::Container, handle: Self::Handle) -> Option<&mut T> {
    c.get_mut(handle)
  }
  fn size(c: &Self::Container) -> usize {
    c.len()
  }
}

impl<T> NoneOverlappingStorage<T> for GenerationalVecStorage {
  fn get_mut_pair(
    c: &mut Self::Container,
    handle: (Self::Handle, Self::Handle),
  ) -> Option<(&mut T, &mut T)> {
    let (a, b) = c.get2_mut(handle.0, handle.1);
    (a?, b?).into()
  }
}

impl<T> HandlePredictableStorage<T> for GenerationalVecStorage {
  fn insert_with(
    c: &mut Self::Container,
    creator: impl FnOnce(Handle<T, Self>) -> T,
  ) -> Handle<T, Self> {
    Handle::new(c.insert_with(|handle| creator(Handle::new(handle))))
  }
}
