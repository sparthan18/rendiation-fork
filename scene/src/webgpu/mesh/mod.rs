use anymap::AnyMap;
use rendiation_renderable_mesh::{
  group::MeshDrawGroup, mesh::IntersectAbleGroupedMesh, GPUMeshData, MeshGPU,
};
use rendiation_webgpu::{GPURenderPass, VertexBufferLayoutOwned, GPU};
use std::{
  any::{Any, TypeId},
  ops::Deref,
};

use rendiation_renderable_mesh::{group::GroupedMesh, mesh::IndexedMesh};
use rendiation_webgpu::VertexBufferSourceType;

pub mod fatline;
pub use fatline::*;

use crate::{
  GPUResourceSceneCache, MeshCell, MeshInner, ResourceLogic, ResourceLogicResult, ResourceMapper,
  ResourceWrapped,
};

pub trait GPUMeshLayoutSupport {
  type VertexInput;
}

impl<I, V, T> GPUMeshLayoutSupport for GroupedMesh<IndexedMesh<I, V, T, Vec<V>>>
where
  V: VertexBufferSourceType,
{
  type VertexInput = Vec<V>;
}

pub trait WebGPUMesh: Any {
  fn setup_pass_and_draw<'a>(
    &self,
    pass: &mut GPURenderPass<'a>,
    group: MeshDrawGroup,
    res: &GPUResourceSceneCache,
  );
  fn update(&mut self, gpu: &GPU, storage: &mut AnyMap, res: &mut GPUResourceSceneCache);
  fn vertex_layout(&self) -> Vec<VertexBufferLayoutOwned>;
  fn topology(&self) -> wgpu::PrimitiveTopology;

  // the reason we use CPS style is for supporting refcell
  fn try_pick(&self, _f: &mut dyn FnMut(&dyn IntersectAbleGroupedMesh)) {}
}

impl GPUResourceSceneCache {
  pub fn update_mesh<M: MeshCPUSource>(
    &mut self,
    m: &mut ResourceWrapped<M>,
    gpu: &GPU,
    storage: &mut AnyMap,
  ) {
    let type_id = TypeId::of::<M>();

    let mapper = self
      .meshes
      .entry(type_id)
      .or_insert_with(|| Box::new(MeshResourceMapper::<M>::default()))
      .downcast_mut::<MeshResourceMapper<M>>()
      .unwrap();
    mapper.get_update_or_insert_with_logic(m, |x| match x {
      ResourceLogic::Create(m) => ResourceLogicResult::Create(m.create(gpu, storage)),
      ResourceLogic::Update(gpu_m, m) => {
        m.update(gpu_m, gpu, storage);
        ResourceLogicResult::Update(gpu_m)
      }
    });
  }

  pub fn setup_mesh<'a, M: MeshCPUSource>(
    &self,
    m: &ResourceWrapped<M>,
    pass: &mut GPURenderPass<'a>,
    group: MeshDrawGroup,
  ) {
    let type_id = TypeId::of::<M>();
    let gpu_m = self
      .meshes
      .get(&type_id)
      .unwrap()
      .downcast_ref::<MeshResourceMapper<M>>()
      .unwrap()
      .get_unwrap(m);

    MeshCPUSource::setup_pass_and_draw(m.deref(), gpu_m, pass, group)
  }
}

type MeshResourceMapper<T> = ResourceMapper<<T as MeshCPUSource>::GPU, T>;
pub trait MeshCPUSource: Any {
  type GPU;
  fn update(&self, gpu_mesh: &mut Self::GPU, gpu: &GPU, storage: &mut AnyMap);
  fn create(&self, gpu: &GPU, storage: &mut AnyMap) -> Self::GPU;
  fn setup_pass_and_draw<'a>(
    &self,
    gpu: &Self::GPU,
    pass: &mut GPURenderPass<'a>,
    group: MeshDrawGroup,
  );
  fn vertex_layout(&self) -> Vec<VertexBufferLayoutOwned>;

  fn topology(&self) -> wgpu::PrimitiveTopology;

  fn try_pick(&self, f: &mut dyn FnMut(&dyn IntersectAbleGroupedMesh));
}

pub struct MeshSource<T> {
  inner: T,
}

impl<T> MeshSource<T> {
  pub fn new(inner: T) -> Self {
    Self {
      inner: inner.into(),
    }
  }
}

impl<T> std::ops::Deref for MeshSource<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}

impl<T> std::ops::DerefMut for MeshSource<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.inner
  }
}

impl<T: IntersectAbleGroupedMesh> IntersectAbleGroupedMesh for MeshSource<T> {
  fn intersect_list(
    &self,
    ray: rendiation_geometry::Ray3,
    conf: &rendiation_renderable_mesh::mesh::MeshBufferIntersectConfig,
    result: &mut rendiation_renderable_mesh::mesh::MeshBufferHitList,
    group: MeshDrawGroup,
  ) {
    self.deref().intersect_list(ray, conf, result, group)
  }

  fn intersect_nearest(
    &self,
    ray: rendiation_geometry::Ray3,
    conf: &rendiation_renderable_mesh::mesh::MeshBufferIntersectConfig,
    group: MeshDrawGroup,
  ) -> rendiation_geometry::Nearest<rendiation_renderable_mesh::mesh::MeshBufferHitPoint> {
    self.deref().intersect_nearest(ray, conf, group)
  }
}

impl<T> MeshCPUSource for MeshSource<T>
where
  T: GPUMeshData + IntersectAbleGroupedMesh + Any,
{
  type GPU = MeshGPU;

  fn update(&self, gpu_mesh: &mut Self::GPU, gpu: &GPU, _: &mut AnyMap) {
    self.deref().update(gpu_mesh, &gpu.device);
  }

  fn create(&self, gpu: &GPU, _: &mut AnyMap) -> Self::GPU {
    self.deref().create(&gpu.device)
  }

  fn setup_pass_and_draw<'a>(
    &self,
    gpu: &Self::GPU,
    pass: &mut GPURenderPass<'a>,
    group: MeshDrawGroup,
  ) {
    gpu.setup_pass(pass);
    gpu.draw(pass, self.get_group(group).into())
  }

  fn vertex_layout(&self) -> Vec<VertexBufferLayoutOwned> {
    self.deref().vertex_layout()
  }

  fn topology(&self) -> wgpu::PrimitiveTopology {
    self.deref().topology()
  }

  fn try_pick(&self, f: &mut dyn FnMut(&dyn IntersectAbleGroupedMesh)) {
    f(self.deref())
  }
}

impl<T: MeshCPUSource + Any> WebGPUMesh for MeshInner<T> {
  fn setup_pass_and_draw<'a>(
    &self,
    pass: &mut GPURenderPass<'a>,
    group: MeshDrawGroup,
    res: &GPUResourceSceneCache,
  ) {
    res.setup_mesh(self, pass, group);
  }

  fn update(&mut self, gpu: &GPU, storage: &mut AnyMap, res: &mut GPUResourceSceneCache) {
    res.update_mesh(self, gpu, storage)
  }

  fn vertex_layout(&self) -> Vec<VertexBufferLayoutOwned> {
    self.deref().vertex_layout()
  }

  fn topology(&self) -> wgpu::PrimitiveTopology {
    self.deref().topology()
  }

  fn try_pick(&self, f: &mut dyn FnMut(&dyn IntersectAbleGroupedMesh)) {
    self.deref().try_pick(f)
  }
}

impl<T: MeshCPUSource + IntersectAbleGroupedMesh + Any> WebGPUMesh for MeshCell<T> {
  fn setup_pass_and_draw<'a>(
    &self,
    pass: &mut GPURenderPass<'a>,
    group: MeshDrawGroup,
    res: &GPUResourceSceneCache,
  ) {
    let inner = self.inner.borrow();
    res.setup_mesh(&inner, pass, group);
  }

  fn update(&mut self, gpu: &GPU, storage: &mut AnyMap, res: &mut GPUResourceSceneCache) {
    let mut inner = self.inner.borrow_mut();
    res.update_mesh(&mut inner, gpu, storage)
  }

  fn vertex_layout(&self) -> Vec<VertexBufferLayoutOwned> {
    let inner = self.inner.borrow();
    inner.vertex_layout()
  }

  fn topology(&self) -> wgpu::PrimitiveTopology {
    self.inner.borrow().topology()
  }

  fn try_pick(&self, f: &mut dyn FnMut(&dyn IntersectAbleGroupedMesh)) {
    let inner = self.inner.borrow();
    inner.try_pick(f);
  }
}
