use crate::{DrawcallHandle, SceneBackend, SceneNodeDataTrait};
use rendiation_math::*;
use rendiation_ral::{ResourceManager, RAL};
use rendiation_render_entity::BoundingData;

pub struct DefaultSceneBackend;

impl<T: RAL> SceneBackend<T> for DefaultSceneBackend {
  type NodeData = SceneNodeData<T>;
  type SceneData = ();
}

pub struct SceneNodeData<T: RAL> {
  pub drawcalls: Vec<DrawcallHandle<T>>,
  pub visible: bool,
  pub net_visible: bool,
  pub render_data: RenderData,
  pub local_matrix: Mat4<f32>,
}

impl<T: RAL> Default for SceneNodeData<T> {
  fn default() -> Self {
    Self::new()
  }
}

impl<T: RAL> SceneNodeDataTrait<T> for SceneNodeData<T> {
  type DrawcallIntoIterType = Vec<DrawcallHandle<T>>;
  fn update_by_parent(
    &mut self,
    parent: Option<&Self>,
    _resource: &mut ResourceManager<T>,
  ) -> bool {
    if let Some(parent) = parent {
      self.render_data.world_matrix = parent.render_data.world_matrix * self.local_matrix;
      self.net_visible = self.visible && parent.net_visible;
    }

    // todo!() update resource

    true
  }
  fn provide_drawcall(&self) -> &Self::DrawcallIntoIterType {
    &self.drawcalls
  }
}

impl<T: RAL> SceneNodeData<T> {
  pub fn new() -> Self {
    Self {
      drawcalls: Vec::new(),
      visible: true,
      net_visible: true,
      render_data: RenderData::new(),
      local_matrix: Mat4::one(),
    }
  }

  pub fn append_drawcall(&mut self, handle: DrawcallHandle<T>) {
    self.drawcalls.push(handle)
  }
}

pub struct RenderData {
  pub world_bounding: Option<BoundingData>,
  pub world_matrix: Mat4<f32>,
  pub normal_matrix: Mat4<f32>,
  pub camera_distance: f32,
}

impl RenderData {
  pub fn new() -> Self {
    Self {
      world_bounding: None,
      world_matrix: Mat4::one(),
      normal_matrix: Mat4::one(),
      camera_distance: 0.,
    }
  }
}
