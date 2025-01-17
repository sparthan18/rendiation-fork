use __core::marker::PhantomData;
use rendiation_shader_api::Std430;

use crate::*;

#[derive(Clone)]
pub struct StorageBufferReadOnlyDataView<T: Std430 + ?Sized> {
  gpu: GPUBufferResourceView,
  phantom: PhantomData<T>,
}

impl<T: Std430 + ?Sized> BindableResourceProvider for StorageBufferReadOnlyDataView<T> {
  fn get_bindable(&self) -> BindingResourceOwned {
    self.gpu.get_bindable()
  }
}
impl<T: Std430 + ?Sized> CacheAbleBindingSource for StorageBufferReadOnlyDataView<T> {
  fn get_binding_build_source(&self) -> CacheAbleBindingBuildSource {
    self.gpu.get_binding_build_source()
  }
}
impl<T: Std430 + ?Sized> BindableResourceView for StorageBufferReadOnlyDataView<T> {
  fn as_bindable(&self) -> gpu::BindingResource {
    self.gpu.as_bindable()
  }
}

impl<T: Std430 + ?Sized> StorageBufferReadOnlyDataView<T> {
  pub fn create(device: &GPUDevice, data: T) -> Self {
    let usage = gpu::BufferUsages::STORAGE | gpu::BufferUsages::COPY_DST;
    let gpu = GPUBuffer::create(device, bytemuck::cast_slice(&[data]), usage);
    let gpu = GPUBufferResource::create_with_raw(gpu, usage).create_default_view();

    Self {
      gpu,
      phantom: PhantomData,
    }
  }
}

/// just short convenient method
pub fn create_storage<T: Std430 + ?Sized>(
  data: T,
  device: impl AsRef<GPUDevice>,
) -> StorageBufferReadOnlyDataView<T> {
  StorageBufferReadOnlyDataView::create(device.as_ref(), data)
}
