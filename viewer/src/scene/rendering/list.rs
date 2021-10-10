use rendiation_webgpu::GPU;

use crate::*;

#[derive(Default)]
pub struct RenderList {
  pub(crate) models: Vec<ModelHandle>,
}

impl RenderList {
  pub fn update(&mut self, scene: &mut Scene, gpu: &GPU, pass: &dyn ViewerRenderPass) {
    if let Some(active_camera) = &mut scene.active_camera {
      let camera_gpu = scene
        .active_camera_gpu
        .get_or_insert_with(|| CameraBindgroup::new(gpu))
        .update(gpu, active_camera, &scene.nodes);

      let mut base = SceneMaterialRenderPrepareCtxBase {
        active_camera,
        camera_gpu,
        pass,
        pipelines: &mut scene.pipeline_resource,
        layouts: &mut scene.layouts,
        textures: &mut scene.texture_2ds,
        texture_cubes: &mut scene.texture_cubes,
        samplers: &mut scene.samplers,
        reference_finalization: &scene.reference_finalization,
      };

      let models = &scene.models;
      self.models.iter().for_each(|handle| {
        let model = models.get(*handle).unwrap();
        let material = scene.materials.get_mut(model.material()).unwrap().as_mut();
        let mesh = scene.meshes.get_mut(model.mesh()).unwrap();
        let node = scene.nodes.get_node_mut(model.node()).data_mut();

        let mut ctx = SceneMaterialRenderPrepareCtx {
          base: &mut base,
          model_info: node.get_model_gpu(gpu).into(),
          active_mesh: mesh.as_ref().into(),
        };

        material.update(gpu, &mut ctx);

        mesh.update(gpu);
      });
    }
  }

  pub fn setup_pass<'p>(
    &self,
    gpu_pass: &mut wgpu::RenderPass<'p>,
    scene: &'p Scene,
    pass: &'p dyn ViewerRenderPass,
  ) {
    let models = &scene.models;

    self.models.iter().for_each(|model| {
      let model = models.get(*model).unwrap();
      model.setup_pass(
        gpu_pass,
        &scene.materials,
        &scene.meshes,
        &scene.nodes,
        scene.active_camera_gpu.as_ref().unwrap(),
        &scene.pipeline_resource,
        pass,
      )
    })
  }
}
