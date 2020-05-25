use crate::{Scene, SceneGraphBackEnd};
use generational_arena::Index;
use rendiation_render_entity::BoundingData;

pub struct RenderObject {
  pub shading_index: Index,
  pub geometry_index: Index,
  pub render_order: i32, // todo for sorting
}

impl RenderObject {
  pub fn get_bounding_local<T: SceneGraphBackEnd>(&self, _scene: &Scene<T>) -> &BoundingData {
    todo!()
  }
}