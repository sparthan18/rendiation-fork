use futures::Stream;
use futures::StreamExt;
use reactive::*;
use tree::CoreTree;
use tree::ParentTree;
use tree::ParentTreeDirty;
use tree::TreeHierarchyDerivedSystem;

use crate::*;

#[derive(Clone)]
pub struct SceneNodeDeriveSystem {
  inner:
    TreeHierarchyDerivedSystem<SceneNodeDerivedData, ParentTreeDirty<SceneNodeDeriveDataDirtyFlag>>,
  updater: Arc<RwLock<StreamCacheUpdateWrapper>>,
  indexed_stream_mapper: Arc<RwLock<SceneNodeChangeStreamIndexMapper>>,
}
type SingleSceneNodeChangeStream = impl Stream<Item = SceneNodeDerivedDataDelta> + Unpin;
type SceneNodeChangeStream = impl Stream<Item = (usize, SceneNodeDerivedDataDelta)> + Unpin;

pub type SceneNodeChangeStreamIndexMapper =
  StreamBroadcaster<SceneNodeChangeStream, SceneNodeDerivedDataDelta, IndexMapping>;

pub type SingleSceneNodeChangeStreamFanOut =
  StreamBroadcaster<SingleSceneNodeChangeStream, SceneNodeDerivedDataDelta, FanOut>;

type StreamCacheUpdate = impl Stream<Item = Vec<IndexedItem<node::SceneNodeDerivedDataDelta>>>
  + Unpin
  + AsRef<StreamVec<SingleSceneNodeChangeStreamFanOut>>;

#[pin_project::pin_project]
struct StreamCacheUpdateWrapper {
  #[pin]
  inner: StreamCacheUpdate,
}

impl Stream for StreamCacheUpdateWrapper {
  type Item = ();

  fn poll_next(
    self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Option<Self::Item>> {
    self.project().inner.poll_next(cx).map(|v| v.map(|_| {}))
  }
}

impl SceneNodeDeriveSystem {
  pub fn new(nodes: &SceneNodeCollection) -> Self {
    let stream = nodes.inner.inner().source.batch_listen();
    let inner_sys = TreeHierarchyDerivedSystem::<
      SceneNodeDerivedData,
      ParentTreeDirty<SceneNodeDeriveDataDirtyFlag>,
    >::new::<ParentTree, _, _, _>(stream, &nodes.inner);

    let indexed_stream_mapper: SceneNodeChangeStreamIndexMapper = inner_sys
      .derived_stream
      .fork_stream()
      .flat_map(futures::stream::iter)
      // we don't care about deletions in this stream
      .filter_map_sync(|d: (usize, Option<SceneNodeDerivedDataDelta>)| d.1.map(|d1| (d.0, d1)))
      .create_index_mapping_broadcaster();

    let indexed_stream_mapper = Arc::new(RwLock::new(indexed_stream_mapper));
    let indexed_stream_mapper_c = indexed_stream_mapper.clone();

    let sub_broad_caster = StreamVec::<SingleSceneNodeChangeStreamFanOut>::default();

    let stream_cache_updating: StreamCacheUpdate = inner_sys
      .derived_stream
      .fork_stream()
      .flat_map(futures::stream::iter)
      .fold_signal_state_stream(sub_broad_caster, move |(idx, delta), sub_broad_caster| {
        if delta.is_none() {
          sub_broad_caster.insert(idx, None)
          // we check if is none first to avoid too much sub stream recreate
        } else if sub_broad_caster.get(idx).is_none() {
          sub_broad_caster.insert(
            idx,
            Some(
              indexed_stream_mapper_c
                .read()
                .unwrap()
                .create_sub_stream_by_index(idx)
                .create_broad_caster(),
            ),
          )
        }
      });

    SceneNodeDeriveSystem {
      inner: inner_sys,
      updater: Arc::new(RwLock::new(StreamCacheUpdateWrapper {
        inner: stream_cache_updating,
      })),
      indexed_stream_mapper,
    }
  }

  pub fn maintain(&mut self) {
    let updater: &mut StreamCacheUpdateWrapper = &mut self.updater.write().unwrap();
    do_updates(updater, |_| {});
    let mut indexed_stream_mapper: &mut SceneNodeChangeStreamIndexMapper =
      &mut self.indexed_stream_mapper.write().unwrap();
    do_updates(&mut indexed_stream_mapper, |_| {});
  }
}

pub type WorldMatrixStream = impl Stream<Item = Mat4<f32>>;

impl SceneNodeDeriveSystem {
  pub fn get_world_matrix(&self, node: &SceneNode) -> Mat4<f32> {
    self.get_world_matrix_by_raw_handle(node.raw_handle().index())
  }

  pub fn get_world_matrix_by_raw_handle(&self, index: usize) -> Mat4<f32> {
    self.inner.visit_derived_tree(|tree| {
      let handle = tree.recreate_handle(index);
      tree.get_node(handle).data().data.world_matrix
    })
  }
  pub fn visit_derived<R>(
    &self,
    index: usize,
    v: impl FnOnce(&SceneNodeDerivedData) -> R,
  ) -> Option<R> {
    self.inner.visit_derived_tree(|tree| {
      let handle = tree.try_recreate_handle(index)?;
      tree.try_get_node(handle).map(|n| &n.data().data).map(v)
    })
  }

  pub fn create_derive_stream(
    &self,
    node: &SceneNode,
  ) -> Option<impl Stream<Item = SceneNodeDerivedDataDelta>> {
    self.create_derived_stream_by_raw_handle(node.raw_handle().index())
  }

  pub fn create_derived_stream_by_raw_handle(
    &self,
    index: usize,
  ) -> Option<impl Stream<Item = SceneNodeDerivedDataDelta>> {
    let derived = self.visit_derived(index, |d| d.clone())?;
    let init_deltas = expand_out(&derived);
    self
      .updater
      .read()
      .unwrap()
      .inner
      .as_ref()
      .get(index)?
      .fork_stream_with_init(init_deltas)
      .into()
  }

  pub fn create_world_matrix_stream(&self, node: &SceneNode) -> Option<WorldMatrixStream> {
    self.create_world_matrix_stream_by_raw_handle(node.raw_handle().index())
  }
  pub fn create_world_matrix_stream_by_raw_handle(
    &self,
    index: usize,
  ) -> Option<WorldMatrixStream> {
    self
      .create_derived_stream_by_raw_handle(index)?
      .filter_map_sync(|d| match d {
        SceneNodeDerivedDataDelta::world_matrix(m) => Some(m),
        SceneNodeDerivedDataDelta::net_visible(_) => None,
      })
      .into()
  }
  pub fn get_net_visible(&self, node: &SceneNode) -> bool {
    self.inner.visit_derived_tree(|tree| {
      let handle = tree.recreate_handle(node.raw_handle().index());
      tree.get_node(handle).data().data.net_visible
    })
  }
}
