use rendiation::*;

use rendiation_marco::BindGroup;
use scene::resource::SceneShading;

pub fn create_block_shading(renderer: &WGPURenderer) -> SceneShading {
  let pipeline = StaticPipelineBuilder::new(
    renderer,
    include_str!("./block.vert"),
    include_str!("./block.frag"),
  )
  .as_mut()
  .binding_group::<BlockShadingParamGroup>()
  .geometry::<StandardGeometry>()
  .to_screen_target()
  .with_default_depth()
  .build();
  SceneShading::new(pipeline)
}

#[derive(BindGroup)]
pub struct BlockShadingParamGroup<'a> {
  #[bind_type = "uniform-buffer:vertex"]
  pub u_mvp_matrix: &'a WGPUBuffer,

  #[bind_type = "texture2d:fragment"]
  pub texture_view: &'a wgpu::TextureView,

  #[bind_type = "sampler:fragment"]
  pub sampler: &'a WGPUSampler,

  #[bind_type = "uniform-buffer:fragment"]
  pub u_camera_world_position: &'a WGPUBuffer,
}
