use rendiation_mesh_buffer::{geometry::*, vertex::Vertex};
use rendiation_webgpu::*;

use render_target::TargetStates;
use rendiation_shadergraph_derives::BindGroup;

use rendiation_shader_library::builtin::*;
use rendiation_shader_library::fog::*;
use rendiation_shader_library::sph::*;
use rendiation_shader_library::transform::*;
use rendiation_shader_library::*;
use transform::CameraTransform;

use rendiation_ral::BindGroupHandle;

#[derive(Shader)]
pub struct BlockShader {
  parameter: BlockShadingParamGroup,
}

impl BlockShader {
  fn create_shader(renderer: &WGPURenderer) -> ShaderGraphBuilder {
    let (mut builder, input) = BlockShader::create_builder(renderer);
    let vertex = builder.vertex_by::<Vertex>();
    let p = input.parameter;

    builder.geometry_by::<IndexedGeometry>();

    let (clip_position, mv_position) = CameraTransform::apply(p.mvp, vertex.position, &builder);

    let frag_normal = builder.set_vary(vertex.normal);
    let frag_uv = builder.set_vary(vertex.uv);
    let frag_mv_position = builder.set_vary(mv_position.xyz());

    let block_color = p.my_texture_view.sample(p.my_sampler, frag_uv);

    let block_color = block_color.xyz() * spherical_harmonics(frag_normal);
    let final_color = FogData::apply_fog(p.fog, block_color, length(frag_mv_position));
    builder.set_frag_output(vec4_31(final_color, builder.c(1.0)));
    builder
  }
}

pub fn create_block_shading(renderer: &WGPURenderer, target: &TargetStates) -> WGPUPipeline {
  BlockShader::create_shader(renderer)
    .create()
    .create_pipeline()
    .as_mut()
    .target_states(target)
    .build(renderer)
}

#[derive(BindGroup)]
pub struct BlockShadingParamGroup {
  #[stage(vert)]
  pub mvp: CameraTransform,

  #[stage(frag)]
  pub fog: FogData,

  #[stage(frag)]
  pub my_texture_view: ShaderGraphTexture,

  #[stage(frag)]
  pub my_sampler: ShaderGraphSampler,
}