use crate::geometry::StandardGeometry;
use crate::texture::Texture;
use rendiation::*;

pub struct TexShading {
  pipeline: WGPUPipeline,

  bindgroup: Option<WGPUBindGroup>,

  // texture_id: usize,
  // matrix_uniform_buffer: WGPUBuffer,
}

impl TexShading {
  pub fn new<R: Renderer>(renderer: &WGPURenderer<R>, texture: Texture) -> Self {
    let mut pipeline_builder = WGPUPipelineDescriptorBuilder::new();
    pipeline_builder
      .vertex_shader(include_str!("./shader.vert"))
      .frag_shader(include_str!("./shader.frag"))
      .binding_group(
        BindGroupLayoutBuilder::new()
          .binding(wgpu::BindGroupLayoutBinding {
            binding: 0,
            visibility: wgpu::ShaderStage::VERTEX,
            ty: wgpu::BindingType::UniformBuffer { dynamic: false },
          })
          .binding(wgpu::BindGroupLayoutBinding {
            binding: 1,
            visibility: wgpu::ShaderStage::FRAGMENT,
            ty: wgpu::BindingType::SampledTexture {
              multisampled: false,
              dimension: wgpu::TextureViewDimension::D2,
            },
          })
          .binding(wgpu::BindGroupLayoutBinding {
            binding: 2,
            visibility: wgpu::ShaderStage::FRAGMENT,
            ty: wgpu::BindingType::Sampler,
          }),
      );

    let pipeline =
      pipeline_builder.build::<StandardGeometry>(&renderer.device, &renderer.swap_chain_descriptor);

    TexShading {
      pipeline,
      bindgroup: None,
    }
  }
}
