pub mod bvh;
pub mod container;
pub mod conversion;
pub mod intersection;
pub mod primitive;

use bytemuck::cast_slice;
pub use container::*;
pub use primitive::*;

pub use bvh::*;
pub use intersection::*;
use rendiation_math_entity::Positioned3D;
use rendiation_ral::{
  GeometryProvider, GeometryResourceInstance, GeometryResourceProvider, ResourceManager, RAL,
};

impl<'a, V, T, U, R> GeometryResourceProvider<R> for IndexedGeometry<V, T, U>
where
  V: Positioned3D + GeometryProvider<R>,
  T: PrimitiveTopology<V>,
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
    let index_buffer = R::create_index_buffer(renderer, cast_slice(&self.index));
    instance.index_buffer = Some(resources.add_index_buffer(index_buffer).index());

    self.data.create_gpu(resources, renderer, &mut instance);
    instance.draw_range = 0..self.get_full_count();
    instance
  }
}
