pub use texture::*;
pub mod texture;

mod uniform;
pub use uniform::*;

mod sampler;
pub use sampler::*;

use crate::*;

pub struct ResourceViewContainer<T: Resource> {
  // when resource view is hold, the resource it self should keep existing
  pub resource: ResourceRc<T>,
  pub view: T::View,
  pub guid: usize,
  pub desc: T::ViewDescriptor,
}

impl<T: Resource> std::ops::Deref for ResourceViewContainer<T> {
  type Target = T::View;

  fn deref(&self) -> &Self::Target {
    &self.view
  }
}

/// store the resource with it's create parameter,
/// and some dropping callbacks
pub struct ResourceContainer<T: Resource> {
  pub guid: usize,
  pub resource: T,
  pub desc: T::Descriptor,
  /// when resource dropped, all referenced bindgroup should drop
  invalidation_tokens: RefCell<Vec<BindGroupCacheInvalidation>>,
}

impl<T: Resource> std::ops::Deref for ResourceContainer<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    &self.resource
  }
}

static RESOURCE_GUID: AtomicUsize = AtomicUsize::new(0);
impl<T: Resource> ResourceContainer<T> {
  pub fn create(desc: T::Descriptor, device: &GPUDevice) -> Self {
    let resource = T::create_resource(&desc, device);
    Self {
      guid: RESOURCE_GUID.fetch_add(1, Ordering::Relaxed),
      resource,
      desc,
      invalidation_tokens: Default::default(),
    }
  }
}

pub trait Resource: 'static {
  type Descriptor;
  type View;
  type ViewDescriptor;

  fn create_resource(des: &Self::Descriptor, device: &GPUDevice) -> Self;
  fn create_view(&self, des: &Self::ViewDescriptor) -> Self::View;
}

pub struct ResourceRc<T: Resource> {
  inner: Rc<ResourceContainer<T>>,
}

impl<T: Resource> std::ops::Deref for ResourceRc<T> {
  type Target = ResourceContainer<T>;

  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}

impl<T: Resource> Clone for ResourceRc<T> {
  fn clone(&self) -> Self {
    Self {
      inner: self.inner.clone(),
    }
  }
}

pub struct ResourceViewRc<T: Resource> {
  inner: Rc<ResourceViewContainer<T>>,
}

impl<T: Resource> std::ops::Deref for ResourceViewRc<T> {
  type Target = ResourceViewContainer<T>;

  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}

impl<T: Resource> Clone for ResourceViewRc<T> {
  fn clone(&self) -> Self {
    Self {
      inner: self.inner.clone(),
    }
  }
}

impl<T> BindableResourceView for ResourceViewRc<T>
where
  T::View: BindableResourceView,
  T: Resource,
{
  fn as_bindable(&self) -> wgpu::BindingResource {
    self.inner.as_bindable()
  }
}

static RESOURCE_VIEW_GUID: AtomicUsize = AtomicUsize::new(0);
impl<T: Resource> ResourceRc<T> {
  #[must_use]
  pub fn create(desc: T::Descriptor, device: &GPUDevice) -> Self {
    Self {
      inner: Rc::new(ResourceContainer::create(desc, device)),
    }
  }

  pub fn create_view(&self, desc: T::ViewDescriptor) -> ResourceViewRc<T> {
    let view = self.inner.resource.create_view(&desc);
    let inner = ResourceViewContainer {
      resource: self.clone(),
      view,
      guid: RESOURCE_VIEW_GUID.fetch_add(1, Ordering::Relaxed),
      desc,
    };
    ResourceViewRc {
      inner: Rc::new(inner),
    }
  }
}

impl<T> BindProvider for ResourceViewRc<T>
where
  T: Resource,
  T::View: BindableResourceView,
{
  fn view_id(&self) -> usize {
    self.inner.guid
  }

  fn add_bind_record(&self, record: BindGroupCacheInvalidation) {
    self
      .inner
      .resource
      .inner
      .invalidation_tokens
      .borrow_mut()
      .push(record);
  }
}
