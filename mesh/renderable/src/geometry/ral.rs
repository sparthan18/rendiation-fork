use rendiation_geometry::Positioned;
use rendiation_ral::*;

use super::{GeometryDataContainer, IndexedGeometry, PrimitiveTopologyMeta};

pub trait RALGeometryDataContainer<T, R>: GeometryDataContainer<T>
where
  T: GeometryProvider,
  R: RAL,
{
  fn create_gpu(
    &self,
    resources: &mut ResourceManager<R>,
    renderer: &mut R::Renderer,
    instance: &mut GeometryResourceInstance<R, T>,
  );
}

impl<R, T> RALGeometryDataContainer<T, R> for Vec<T>
where
  R: RAL,
  T: GeometryProvider + Clone + VertexBufferLayoutProvider + bytemuck::Pod,
{
  fn create_gpu(
    &self,
    resources: &mut ResourceManager<R>,
    renderer: &mut R::Renderer,
    instance: &mut GeometryResourceInstance<R, T>,
  ) {
    let vertex_buffer =
      R::create_vertex_buffer(renderer, bytemuck::cast_slice(self.as_ref()), T::DESCRIPTOR);
    instance.vertex_buffers = vec![resources.add_vertex_buffer(vertex_buffer).index()];
  }
}

impl<'a, V, T, U, R> GeometryResourceCreator<R> for IndexedGeometry<u16, V, T, U>
where
  V: Positioned<f32, 3> + GeometryProvider,
  T: PrimitiveTopologyMeta<V>,
  U: RALGeometryDataContainer<V, R> + 'static,
  R: RAL,
{
  type Instance = GeometryResourceInstance<R, V>;

  fn create(
    &self,
    resources: &mut ResourceManager<R>,
    renderer: &mut R::Renderer,
  ) -> Self::Instance {
    let mut instance = GeometryResourceInstance::new();
    let index_buffer = R::create_index_buffer(renderer, bytemuck::cast_slice(&self.index));
    instance.index_buffer = Some(resources.add_index_buffer(index_buffer).index());

    self.data.create_gpu(resources, renderer, &mut instance);
    instance.draw_range = 0..self.index.len() as u32;
    instance
  }
}

impl<V, T, U, R> GeometryResourceInstanceCreator<R, V> for IndexedGeometry<u16, V, T, U>
where
  V: Positioned<f32, 3> + GeometryProvider,
  T: PrimitiveTopologyMeta<V>,
  U: RALGeometryDataContainer<V, R> + 'static,
  R: RAL,
{
}

impl<'a, V, T, U> VertexBufferLayoutGroupProvider for IndexedGeometry<u16, V, T, U>
where
  V: Positioned<f32, 3> + VertexBufferLayoutProvider,
  T: PrimitiveTopologyMeta<V>,
  U: GeometryDataContainer<V>,
{
  fn create_descriptor() -> Vec<VertexBufferLayout<'static>> {
    todo!()
    // VertexStateDescriptor {
    //   index_format: IndexFormat::Uint16,
    //   vertex_buffers: &[V::DESCRIPTOR],
    // }
  }
}

impl<'a, V, T, U> GeometryDescriptorProvider for IndexedGeometry<u16, V, T, U>
where
  V: Positioned<f32, 3> + VertexBufferLayoutProvider,
  T: PrimitiveTopologyMeta<V>,
  U: GeometryDataContainer<V>,
{
  fn get_primitive_topology() -> rendiation_ral::PrimitiveTopologyMeta {
    T::ENUM
  }
}
