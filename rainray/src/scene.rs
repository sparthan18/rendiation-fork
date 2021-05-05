use crate::{background::*, NormalizedVec3, RainrayMaterial, Vec3};
use crate::{light::*, Intersection, PossibleIntersection};
use crate::{model::*, RainRayGeometry};
use arena_tree::NextTraverseVisit;
use rendiation_algebra::*;
use rendiation_geometry::Ray3;
use sceno::SceneBackend;

pub struct RainrayScene;

impl SceneBackend for RainrayScene {
  type Model = Box<dyn RainrayModel>;
  type Material = Box<dyn RainrayMaterial>;
  type Mesh = Box<dyn RainRayGeometry>;
  type Background = Box<dyn Background>;
  type Light = Box<dyn Light>;
}

pub type Scene = sceno::Scene<RainrayScene>;
pub type SceneNode = sceno::SceneNode<RainrayScene>;
pub type NodeHandle = sceno::SceneNodeHandle<RainrayScene>;
pub type ModelHandle = sceno::ModelHandle<RainrayScene>;
pub type MeshHandle = sceno::MeshHandle<RainrayScene>;
pub type MaterialHandle = sceno::MaterialHandle<RainrayScene>;

pub struct ModelInstance<'a> {
  pub node: &'a SceneNode,
  pub matrix_world_inverse: Mat4<f32>,
  pub normal_matrix_inverse: Mat4<f32>,
  pub model: &'a dyn RainrayModel,
}

impl<'a> ModelInstance<'a> {
  pub fn sample(
    &self,
    view_dir: NormalizedVec3,
    intersection: &Intersection,
    scene: &RayTraceScene<'a>,
  ) -> BSDFSampleResult {
    // todo do space conversion
    self.model.sample(view_dir, intersection, scene)
  }

  pub fn bsdf(
    &self,
    view_dir: NormalizedVec3,
    light_dir: NormalizedVec3,
    intersection: &Intersection,
    scene: &RayTraceScene<'a>,
  ) -> Vec3 {
    self.model.bsdf(view_dir, light_dir, intersection, scene)
  }
}

pub struct LightInstance<'a> {
  pub node: &'a SceneNode,
  pub light: &'a dyn Light,
}

pub struct RayTraceScene<'a> {
  pub scene: &'a Scene,
  pub lights: Vec<LightInstance<'a>>,
  pub models: Vec<ModelInstance<'a>>,
}

impl<'a> RayTraceScene<'a> {
  pub fn get_min_dist_hit(&self, mut ray: Ray3) -> Option<(Intersection, f32, &ModelInstance)> {
    let mut min_distance = std::f32::INFINITY;
    let mut result = None;
    for model_instance in &self.models {
      let ModelInstance {
        model,
        matrix_world_inverse,
        normal_matrix_inverse,
        ..
      } = model_instance;

      let ray_world = ray;
      ray.apply_matrix(*matrix_world_inverse);

      if let PossibleIntersection(Some(mut intersection)) = model.intersect(ray, self) {
        intersection.apply_matrix(*matrix_world_inverse, *normal_matrix_inverse);
        let distance = intersection.position.distance(ray_world.origin);

        if distance < min_distance {
          intersection.adjust_hit_position();
          min_distance = distance;
          result = Some((intersection, distance, model_instance))
        }
      }
    }
    result
  }
  pub fn test_point_visible_to_point(&self, point_a: Vec3, point_b: Vec3) -> bool {
    let ray = Ray3::from_point_to_point(point_a, point_b);
    let distance = (point_a - point_b).length();

    if let Some(hit_result) = self.get_min_dist_hit(ray) {
      hit_result.1 > distance
    } else {
      true
    }
  }
}

pub trait RainraySceneExt {
  fn convert(&self) -> RayTraceScene;
}

impl RainraySceneExt for Scene {
  fn convert(&self) -> RayTraceScene {
    let scene_light = &self.lights;
    let scene_model = &self.models;

    let mut lights = Vec::new();
    let mut models = Vec::new();

    let root = self.get_root_handle();
    self
      .nodes
      .traverse_immutable(root, &mut Vec::new(), |this, _| {
        let node_data = this.data();
        node_data.payload.iter().for_each(|payload| match payload {
          sceno::SceneNodePayload::Model(model) => {
            let model = scene_model.get(*model).unwrap().as_ref();
            let matrix_world_inverse = node_data.world_matrix.inverse_or_identity();
            models.push(ModelInstance {
              node: node_data,
              matrix_world_inverse,
              normal_matrix_inverse: matrix_world_inverse.transpose(),
              model,
            });
          }
          sceno::SceneNodePayload::Light(light) => {
            let light = scene_light.get(*light).unwrap().as_ref();
            lights.push(LightInstance {
              node: node_data,
              light,
            });
          }
        });
        NextTraverseVisit::VisitChildren
      });

    RayTraceScene {
      scene: self,
      lights,
      models,
    }
  }
}
