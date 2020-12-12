use super::ShaderBindableResourceManager;
use crate::{BindGroupCreator, BindGroupHandle, BindGroupProvider, ResourceManager, RAL};
use arena::{Arena, Handle};
use std::{any::Any, collections::HashSet};

pub struct BindGroupManager<R: RAL> {
  storage: Arena<Box<dyn BindgroupStorageTrait<R>>>,
  modified: HashSet<Handle<Box<dyn BindgroupStorageTrait<R>>>>,
}

impl<R: RAL> BindGroupManager<R> {
  pub fn new() -> Self {
    Self {
      storage: Arena::new(),
      modified: HashSet::new(),
    }
  }

  pub fn maintain_gpu(
    &mut self,
    renderer: &R::Renderer,
    resources: &ShaderBindableResourceManager<R>,
  ) {
    let storage = &mut self.storage;
    self.modified.drain().for_each(|d| {
      storage.get_mut(d).map(|bp| {
        bp.maintain_gpu(renderer, resources);
      });
    })
  }

  pub fn get_bindgroup_boxed<T: BindGroupProvider<R>>(
    &self,
    handle: BindGroupHandle<R, T>,
  ) -> &dyn BindgroupStorageTrait<R> {
    let handle = unsafe { handle.cast_type() };
    self.storage.get(handle).unwrap().as_ref()
  }

  pub fn get_bindgroup_unwrap<T: BindGroupProvider<R>>(
    &self,
    handle: BindGroupHandle<R, T>,
  ) -> &<T as BindGroupProvider<R>>::Instance {
    let handle = unsafe { handle.cast_type() };
    let storage = self.storage.get(handle).unwrap();
    let storage = storage
      .as_any()
      .downcast_ref::<BindgroupPair<R, T>>()
      .unwrap();
    &storage.data
  }

  pub fn get_gpu<T: BindGroupProvider<R>>(&self, handle: BindGroupHandle<R, T>) -> &R::BindGroup {
    let handle = unsafe { handle.cast_type() };
    self.storage.get(handle).unwrap().get_gpu()
  }

  pub fn add<T: BindGroupCreator<R>>(&mut self, bindgroup: T::Instance) -> BindGroupHandle<R, T> {
    let pair = BindgroupPair::<R, T> {
      data: bindgroup,
      gpu: None,
    };
    let handle = self.storage.insert(Box::new(pair));
    self.modified.insert(handle);
    unsafe { handle.cast_type() }
  }

  // put updated handle into modified list, and return the instance for others to modify
  pub fn update<T: BindGroupProvider<R>>(
    &mut self,
    handle: BindGroupHandle<R, T>,
  ) -> &mut T::Instance {
    let handle = unsafe { handle.cast_type() };
    self.modified.insert(handle);
    let pair = self.storage.get_mut(handle).unwrap();
    pair
      .as_any_mut()
      .downcast_mut::<BindgroupPair<R, T>>()
      .unwrap()
      .update()
  }

  pub fn delete<T: BindGroupProvider<R>>(&mut self, handle: BindGroupHandle<R, T>) {
    let handle = unsafe { handle.cast_type() };
    self.modified.remove(&handle);
    self.storage.remove(handle);
  }
}

pub trait BindgroupStorageTrait<R: RAL>: Any {
  fn maintain_gpu(&mut self, renderer: &R::Renderer, resources: &ShaderBindableResourceManager<R>);
  fn get_gpu(&self) -> &R::BindGroup;
  fn as_any(&self) -> &dyn Any;
  fn as_any_mut(&mut self) -> &mut dyn Any;
  fn apply(
    // maybe we can use unwrap downcast
    &self,
    render_pass: &mut R::RenderPass,
    resources: &ResourceManager<R>,
    index: usize,
    shading: &R::Shading,
  );
}

impl<R: RAL, T: BindGroupCreator<R>> BindgroupStorageTrait<R> for BindgroupPair<R, T> {
  fn maintain_gpu<'a>(
    &mut self,
    renderer: &R::Renderer,
    resources: &ShaderBindableResourceManager<R>,
  ) {
    self.gpu = Some(T::create_bindgroup(&self.data, renderer, resources));
  }
  fn get_gpu(&self) -> &R::BindGroup {
    self.gpu.as_ref().unwrap()
  }
  fn as_any(&self) -> &dyn Any {
    self
  }
  fn as_any_mut(&mut self) -> &mut dyn Any {
    self
  }
  fn apply(
    &self,
    render_pass: &mut R::RenderPass,
    resources: &ResourceManager<R>,
    index: usize,
    shading: &R::Shading,
  ) {
    T::apply(
      &self.data,
      self.gpu.as_ref().unwrap(),
      index,
      shading,
      &resources.bindable,
      render_pass,
    );
  }
}

pub struct BindgroupPair<R: RAL, T: BindGroupProvider<R>> {
  data: T::Instance,
  gpu: Option<R::BindGroup>,
}

impl<R: RAL, T: BindGroupProvider<R>> BindgroupPair<R, T> {
  fn update(&mut self) -> &mut T::Instance {
    &mut self.data
  }
}
