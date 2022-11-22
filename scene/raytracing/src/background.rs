use rendiation_algebra::*;
use rendiation_geometry::Ray3;
use rendiation_scene_core::{SceneBackGround, SolidBackground};

pub trait RayTracingBackground: Send + Sync + 'static + dyn_clone::DynClone {
  fn sample(&self, ray: &Ray3) -> Vec3<f32>;
  fn into_scene_background(&self) -> Option<SceneBackGround> {
    None
  }
}

dyn_clone::clone_trait_object!(RayTracingBackground);

impl RayTracingBackground for SceneBackGround {
  fn sample(&self, ray: &Ray3) -> Vec3<f32> {
    match self {
      SceneBackGround::Solid(s) => s.sample(ray),
      SceneBackGround::Env(s) => {
        // todo
        Vec3::zero()
      }
      SceneBackGround::Foreign(_) => todo!(),
    }
  }
  fn into_scene_background(&self) -> Option<SceneBackGround> {
    self.clone().into()
  }
}

impl RayTracingBackground for SolidBackground {
  fn sample(&self, _ray: &Ray3) -> Vec3<f32> {
    self.intensity
  }
  fn into_scene_background(&self) -> Option<SceneBackGround> {
    SceneBackGround::Solid(*self).into()
  }
}

impl RayTracingBackground for GradientBackground {
  fn sample(&self, ray: &Ray3) -> Vec3<f32> {
    let t = ray.direction.y / 2.0 + 1.;
    self.bottom_intensity.lerp(self.top_intensity, t)
  }
}

#[derive(Clone)]
pub struct GradientBackground {
  pub top_intensity: Vec3<f32>,
  pub bottom_intensity: Vec3<f32>,
}
