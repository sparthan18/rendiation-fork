use ::reactive::EventSource;

use crate::*;

pub struct ReactiveTreeCollection<T, X: IncrementalBase> {
  pub inner: T,
  pub source: EventSource<TreeMutation<X>>,
}

impl<T: Default, X: IncrementalBase> Default for ReactiveTreeCollection<T, X> {
  fn default() -> Self {
    Self {
      inner: Default::default(),
      source: Default::default(),
    }
  }
}

impl<T, X, N> CoreTree for ReactiveTreeCollection<T, X>
where
  T: CoreTree<Handle = TreeNodeHandle<N>, Node = N>,
  N: std::ops::Deref<Target = X>,
  X: IncrementalBase + Clone,
{
  type Node = T::Node;
  type Handle = TreeNodeHandle<T::Node>;
  fn try_recreate_handle(&self, index: usize) -> Option<Self::Handle> {
    self.inner.try_recreate_handle(index)
  }

  fn node_has_parent(&self, handle: Self::Handle) -> bool {
    self.inner.node_has_parent(handle)
  }

  fn get_node_data(&self, handle: Self::Handle) -> &Self::Node {
    self.inner.get_node_data(handle)
  }

  fn get_node_data_mut(&mut self, handle: Self::Handle) -> &mut Self::Node {
    // mutation should emit by hand
    self.inner.get_node_data_mut(handle)
  }

  fn create_node(&mut self, data: Self::Node) -> Self::Handle {
    let d = data.deref().clone();
    let handle = self.inner.create_node(data);
    self.source.emit(&TreeMutation::Create {
      data: d,
      node: handle.index(),
    });
    handle
  }

  fn delete_node(&mut self, handle: Self::Handle) {
    self.source.emit(&TreeMutation::Delete(handle.index()));
    self.inner.delete_node(handle)
  }

  fn node_add_child_by(
    &mut self,
    parent: Self::Handle,
    child_to_attach: Self::Handle,
  ) -> Result<(), TreeMutationError> {
    if !self.inner.node_has_parent(child_to_attach) {
      self.source.emit(&TreeMutation::Attach {
        parent_target: parent.index(),
        node: child_to_attach.index(),
      });
    }
    self.inner.node_add_child_by(parent, child_to_attach)
  }

  fn node_detach_parent(&mut self, child_to_detach: Self::Handle) -> Result<(), TreeMutationError> {
    if self.inner.node_has_parent(child_to_detach) {
      self.source.emit(&TreeMutation::Detach {
        node: child_to_detach.index(),
      });
    }
    self.inner.node_detach_parent(child_to_detach)
  }
}

impl<T, X, N> ShareCoreTree for ReactiveTreeCollection<T, X>
where
  T: ShareCoreTree<Handle = TreeNodeHandle<N>, Node = N>,
  N: std::ops::Deref<Target = X>,
  X: IncrementalBase + Clone,
{
  type Node = T::Node;
  type Handle = TreeNodeHandle<T::Node>;
  type Core = T::Core;
  fn visit_core_tree<R>(&self, v: impl FnOnce(&Self::Core) -> R) -> R {
    self.inner.visit_core_tree(v)
  }

  fn recreate_handle(&self, index: usize) -> Self::Handle {
    self.inner.recreate_handle(index)
  }

  fn node_has_parent(&self, handle: Self::Handle) -> bool {
    self.inner.node_has_parent(handle)
  }

  fn visit_node_data<R>(&self, handle: Self::Handle, v: impl FnOnce(&Self::Node) -> R) -> R {
    self.inner.visit_node_data(handle, v)
  }

  fn mutate_node_data<R>(&self, handle: Self::Handle, v: impl FnOnce(&mut Self::Node) -> R) -> R {
    self.inner.mutate_node_data(handle, v)
  }

  fn create_node(&self, data: Self::Node) -> Self::Handle {
    let d = data.deref().clone();
    let handle = self.inner.create_node(data);
    self.source.emit(&TreeMutation::Create {
      data: d,
      node: handle.index(),
    });
    handle
  }

  fn delete_node(&self, handle: Self::Handle) {
    self.source.emit(&TreeMutation::Delete(handle.index()));
    self.inner.delete_node(handle)
  }

  fn node_add_child_by(
    &self,
    parent: Self::Handle,
    child_to_attach: Self::Handle,
  ) -> Result<(), TreeMutationError> {
    // to prevent emit invalid event
    if !self.node_has_parent(child_to_attach) {
      self.source.emit(&TreeMutation::Attach {
        parent_target: parent.index(),
        node: child_to_attach.index(),
      });
    }
    self.inner.node_add_child_by(parent, child_to_attach)
  }

  fn node_detach_parent(&self, child_to_detach: Self::Handle) -> Result<(), TreeMutationError> {
    // to prevent emit invalid event
    if self.node_has_parent(child_to_detach) {
      self.source.emit(&TreeMutation::Detach {
        node: child_to_detach.index(),
      });
    }
    self.inner.node_detach_parent(child_to_detach)
  }
}
