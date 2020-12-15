use rendiation_math::{Vec2, Vector};
use rendiation_math_entity::Ray3;
use rendiation_render_entity::{
  color::{Color, LinearRGBColorSpace},
  Camera, Raycaster,
};

use super::Integrator;
use crate::{math::rand, math::Vec3, ray::Intersection, scene::Scene, LightSampleResult, Material};
use rendiation_math::Zero;

pub struct PathTraceIntegrator {
  pub exposure_upper_bound: f32,
  pub trace_fix_sample_count: u64,
  pub bounce_time_limit: u64,
  pub roulette_threshold: f32,
  pub roulette_factor: f32,
}

impl Default for PathTraceIntegrator {
  fn default() -> Self {
    Self {
      exposure_upper_bound: 1.0,
      bounce_time_limit: 20,
      trace_fix_sample_count: 200,
      roulette_threshold: 0.05,
      roulette_factor: 0.05,
    }
  }
}

impl PathTraceIntegrator {
  pub fn path_trace(&self, ray: &Ray3, scene: &Scene) -> Vec3 {
    let mut energy = Vec3::new(0., 0., 0.);
    let mut throughput = Vec3::new(1., 1., 1.);
    let mut current_ray = *ray;

    for _depth in 0..self.bounce_time_limit {
      let hit_result = scene.get_min_dist_hit(&current_ray);

      // hit outside scene, sample background;
      if hit_result.is_none() {
        energy += scene.env.sample(&current_ray) * throughput;
        break;
      }

      let (intersection, model) = hit_result.unwrap();
      let material = &model.material;

      if let Some(scatter) = material.scatter(&current_ray.direction, &intersection) {
        if scatter.pdf == 0.0 {
          break;
        }

        let next_ray = scatter.create_next_ray(intersection.hit_position);

        energy += material.sample_emissive(&intersection) * throughput;
        energy += self.sample_lights(scene, material.as_ref(), &intersection, &next_ray.direction)
          * throughput;

        let cos = scatter.out_dir.dot(intersection.hit_normal).abs();
        let bsdf = material.bsdf(&current_ray.direction, &next_ray.direction, &intersection);
        throughput = throughput * cos * bsdf / scatter.pdf;

        // roulette exist
        if throughput.max_channel() < self.roulette_threshold {
          if (rand() < self.roulette_factor) {
            break;
          }
          throughput /= 1. - self.roulette_factor;
        }

        current_ray = next_ray;
      } else {
        break;
      }
    }

    energy
  }

  // next event estimation
  fn sample_lights(
    &self,
    scene: &Scene,
    material: &dyn Material,
    intersection: &Intersection,
    light_out_dir: &Vec3,
  ) -> Vec3 {
    let mut energy = Vec3::new(0.0, 0.0, 0.0);
    for light in &scene.lights {
      if let Some(LightSampleResult {
        emissive,
        light_in_dir,
      }) = light.sample(intersection.hit_position, scene)
      {
        let bsdf = material.bsdf(&light_in_dir, light_out_dir, intersection);
        energy += bsdf * emissive * -light_in_dir.dot(intersection.hit_normal);
      }
    }
    energy
  }
}

impl Integrator for PathTraceIntegrator {
  fn integrate(
    &self,
    camera: &Camera,
    scene: &Scene,
    frame_size: Vec2<usize>,
    current: Vec2<usize>,
  ) -> Color<LinearRGBColorSpace<f32>> {
    let mut pixel_left_top = current.map(|v| v as f32) / frame_size.map(|v| v as f32);
    pixel_left_top.y = 1.0 - pixel_left_top.y;

    let jitter_size = frame_size.map(|v| 1.0 / v as f32);

    let mut energy_acc = Vec3::zero();

    for _ in 0..self.trace_fix_sample_count {
      let ray = camera.create_screen_ray(pixel_left_top + jitter_size.map(|v| v * rand()));
      energy_acc += self.path_trace(&ray, scene);
    }

    let energy_max = self.trace_fix_sample_count as f32 * self.exposure_upper_bound;
    Color::new(energy_acc / energy_max)
  }
}
