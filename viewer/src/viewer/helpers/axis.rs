use rendiation_algebra::*;
use rendiation_renderable_mesh::tessellation::{CylinderMeshParameter, IndexedMeshTessellator};

use crate::*;

pub struct AxisHelper {
  pub enabled: bool,
  x: MeshModel,
  y: MeshModel,
  z: MeshModel,
}

impl PassContent for AxisHelper {
  fn update(
    &mut self,
    gpu: &rendiation_webgpu::GPU,
    scene: &mut Scene,
    _resource: &mut ResourcePoolInner,
    pass_info: &PassTargetFormatInfo,
  ) {
    if let Some(active_camera) = &mut scene.active_camera {
      let (active_camera, camera_gpu) = active_camera.get_updated_gpu(gpu);

      let mut base = SceneMaterialRenderPrepareCtxBase {
        active_camera,
        camera_gpu,
        pass: pass_info,
        resources: &mut scene.resources,
      };

      self.x.update(gpu, &mut base);
      self.y.update(gpu, &mut base);
      self.z.update(gpu, &mut base);
    }
  }

  fn setup_pass<'a>(
    &'a self,
    pass: &mut rendiation_webgpu::GPURenderPass<'a>,
    scene: &'a Scene,
    pass_info: &'a PassTargetFormatInfo,
  ) {
    self.x.setup_pass(
      pass,
      scene.active_camera.as_ref().unwrap().expect_gpu(),
      &scene.resources,
      pass_info,
    );

    self.y.setup_pass(
      pass,
      scene.active_camera.as_ref().unwrap().expect_gpu(),
      &scene.resources,
      pass_info,
    );

    self.z.setup_pass(
      pass,
      scene.active_camera.as_ref().unwrap().expect_gpu(),
      &scene.resources,
      pass_info,
    );
  }
}

fn material(color: Vec3<f32>) -> impl Material {
  let mut material = FlatMaterial {
    color,
    states: Default::default(),
  };
  material.states.depth_write_enabled = false;
  material.states.depth_compare = wgpu::CompareFunction::Always;
  MaterialCell::new(material)
}

impl AxisHelper {
  pub fn new(scene: &mut Scene) -> Self {
    let cylinder = CylinderMeshParameter {
      radius_top: 0.01,
      radius_bottom: 0.01,
      height: 4.,
      ..Default::default()
    }
    .tessellate();
    let cylinder = MeshCell::new(cylinder);

    // let tip = SphereMeshParameter::default().tessellate();
    // let tip = MeshCell::new(mesh);

    let x_node = scene.root.create_child();
    x_node.mutate(|node| {
      node.local_matrix = Mat4::lookat(
        Vec3::new(-1., 0., 0.),
        Vec3::splat(0.),
        Vec3::new(0., 1., 0.),
      );
    });
    let x = MeshModel::new(material(Vec3::new(1., 0., 0.)), cylinder.clone(), x_node);

    let y_node = scene.root.create_child();
    y_node.mutate(|node| {
      node.local_matrix = Mat4::lookat(
        Vec3::new(0., -1., 0.),
        Vec3::splat(0.),
        Vec3::new(1., 0., 0.),
      );
    });
    let y = MeshModel::new(material(Vec3::new(0., 1., 0.)), cylinder.clone(), y_node);

    let z_node = scene.root.create_child();
    z_node.mutate(|node| {
      node.local_matrix = Mat4::lookat(
        Vec3::new(0., 0., -1.),
        Vec3::splat(0.),
        Vec3::new(0., 1., 0.),
      );
    });
    let z = MeshModel::new(material(Vec3::new(0., 0., 1.)), cylinder, z_node);

    Self {
      enabled: true,
      x,
      y,
      z,
    }
  }
}
