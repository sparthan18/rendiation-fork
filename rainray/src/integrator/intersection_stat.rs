use crate::{Integrator, RayTraceable};
use rendiation_color::LinearRGBColor;

pub struct IntersectionVisualize {
  pub box_weight: f32,
  pub sphere_weight: f32,
  pub triangle_weight: f32,

  pub weight_bound: f32,
}

impl Default for IntersectionVisualize {
  fn default() -> Self {
    Self {
      box_weight: 1.,
      sphere_weight: 1.,
      triangle_weight: 1.,
      weight_bound: 150.,
    }
  }
}

impl<T: RayTraceable> Integrator<T> for IntersectionVisualize {
  fn integrate(&self, target: &T, ray: rendiation_geometry::Ray3) -> LinearRGBColor<f32> {
    let stat = target.get_min_dist_hit_stat(ray);
    let cost_estimate = self.box_weight * stat.box3 as f32
      + self.sphere_weight * stat.sphere as f32
      + self.triangle_weight * stat.triangle as f32;

    LinearRGBColor::splat(cost_estimate / self.weight_bound)
  }
}
