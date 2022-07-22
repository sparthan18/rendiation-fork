use std::{cell::RefCell, rc::Rc};

use interphaser::{
  lens, mouse, mouse_move,
  winit::event::{ElementState, MouseButton},
  Component, Lens,
};
use rendiation_algebra::{Mat4, Vec3};
// use rendiation_geometry::{OptionalNearest, Ray3};
// use rendiation_renderable_mesh::{
//   mesh::{MeshBufferHitPoint, MeshBufferIntersectConfig},
//   tessellation::{CubeMeshParameter, IndexedMeshTessellator},
// };

use crate::{
  helpers::axis::{solid_material, Arrow},
  *,
};

/// Gizmo is a useful widget in 3d design/editor software.
/// User could use this to modify the scene node's transformation.
///
pub struct Gizmo {
  states: GizmoState,
  root: SceneNode,
  target: Option<SceneNode>,
  auto_scale: Rc<RefCell<ViewAutoScalable>>,
  view: Component3DCollection<GizmoState>,
}

impl Gizmo {
  pub fn new(parent: &SceneNode) -> Self {
    let root = &parent.create_child();
    let auto_scale = ViewAutoScalable {
      override_position: ViewAutoScalablePositionOverride::SyncNode(root.clone()),
      independent_scale_factor: 100.,
    };
    let auto_scale = &Rc::new(RefCell::new(auto_scale));
    let x = build_axis_arrow(root, auto_scale)
      .toward_x()
      .eventable::<GizmoState>()
      .update(|s, arrow| arrow.root.set_visible(s.show_x()))
      .on(active(lens!(GizmoState, active.x)));

    let y = build_axis_arrow(root, auto_scale)
      .toward_y()
      .eventable::<GizmoState>()
      .update(|s, arrow| arrow.root.set_visible(s.show_y()))
      .on(active(lens!(GizmoState, active.y)));

    let z = build_axis_arrow(root, auto_scale)
      .toward_z()
      .eventable::<GizmoState>()
      .update(|s, arrow| arrow.root.set_visible(s.show_z()))
      .on(active(lens!(GizmoState, active.z)));

    let view = collection3d().with(x).with(y).with(z);

    Self {
      states: Default::default(),
      root: root.clone(),
      auto_scale: auto_scale.clone(),
      view,
      target: None,
    }
  }

  pub fn set_target(&mut self, target: Option<SceneNode>) {
    self.target = target;
  }

  pub fn event(&mut self, event: &mut EventCtx3D) {
    if self.target.is_none() {
      return;
    }

    // dispatch 3d events into 3d components, handling state active
    self.view.event(&mut self.states, event);

    // after active states get updated, we handling mouse moving in gizmo level
    if mouse_move(event.raw_event).is_some() {
      if !self.states.active.has_active() {
        return;
      }
      // let target_world = self.root.get_world_matrix();

      if self.states.active.only_x() {
        //
      }
      if self.states.active.only_y() {
        //
      }
      if self.states.active.only_z() {
        //
      }
    }
  }
  pub fn update(&mut self) {
    if self.target.is_none() {
      return;
    }

    let mut ctx = UpdateCtx3D { placeholder: &() };

    self.view.update(&self.states, &mut ctx);

    self.root.set_local_matrix(Mat4::translate(1., 0., 1.));
  }
}

// this logic mixed with click state handling, try separate it
fn active(active: impl Lens<GizmoState, bool>) -> impl FnMut(&mut GizmoState, &EventCtx3D) {
  let mut is_mouse_down = false;
  move |state, event| {
    if let Some(event3d) = &event.event_3d {
      match event3d {
        Event3D::MouseDown { world_position } => {
          is_mouse_down = true;
          if active.with(state, |active| *active) {
            state.last_active_world_position = *world_position;
          }
        }
        Event3D::MouseUp { .. } => {
          if is_mouse_down {
            active.with_mut(state, |active| {
              *active = true;
            });
          }
        }
        _ => {}
      }
    }

    if let Some((MouseButton::Left, ElementState::Released)) = mouse(event.raw_event) {
      is_mouse_down = false;
    }
  }
}

impl PassContentWithCamera for &mut Gizmo {
  fn render(&mut self, pass: &mut SceneRenderPass, camera: &SceneCamera) {
    if self.target.is_none() {
      return;
    }

    let dispatcher = &pass.default_dispatcher();
    self.view.render(pass, dispatcher, camera)
  }
}

// fn build_box() -> Box<dyn SceneRenderable> {
//   let mesh = CubeMeshParameter::default().tessellate();
//   let mesh = MeshCell::new(MeshSource::new(mesh));
//   todo!();
// }

// fn build_rotation_circle() -> Box<dyn SceneRenderable> {
//   let mut position = Vec::new();
//   let segments = 50;
//   for i in 0..segments {
//     let p = i as f32 / segments as f32;
//     position.push(Vec3::new(p.cos(), p.sin(), 0.))
//   }
//   todo!();
// }

fn build_axis_arrow(root: &SceneNode, auto_scale: &Rc<RefCell<ViewAutoScalable>>) -> Arrow {
  let (cylinder, tip) = Arrow::default_shape();
  let (cylinder, tip) = (&cylinder, &tip);
  let material = &solid_material((0.8, 0.1, 0.1));
  Arrow::new_reused(root, auto_scale, material, cylinder, tip)
}

#[derive(Default)]
struct GizmoState {
  active: AxisActiveState,
  last_active_world_position: Vec3<f32>,
}

impl GizmoState {
  fn show_x(&self) -> bool {
    !self.active.has_active() || self.active.x
  }
  fn show_y(&self) -> bool {
    !self.active.has_active() || self.active.y
  }
  fn show_z(&self) -> bool {
    !self.active.has_active() || self.active.z
  }
}

#[derive(Copy, Clone, Default)]
pub struct AxisActiveState {
  x: bool,
  y: bool,
  z: bool,
}

impl AxisActiveState {
  pub fn reset(&mut self) {
    *self = Default::default();
  }

  pub fn has_active(&self) -> bool {
    self.x && self.y && self.z
  }
  pub fn only_x(&self) -> bool {
    self.x && !self.y && !self.z
  }
  pub fn only_y(&self) -> bool {
    !self.x && self.y && !self.z
  }
  pub fn only_z(&self) -> bool {
    !self.x && !self.y && self.z
  }
}
