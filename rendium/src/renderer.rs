use rendiation::geometry::quad_maker;
use rendiation::*;
use rendiation_math::Vec4;
use rendiation_render_entity::*;

pub struct GUIRenderer {
  quad: StandardGeometry,
  view: Vec4<f32>,
  camera: OrthographicCamera,
  camera_gpu_buffer: WGPUBuffer,
  canvas: WGPUTexture,
  quad_pipeline: WGPUPipeline,
}

impl GUIRenderer {
  pub fn new(renderer: &WGPURenderer, size: (f32, f32)) -> Self {
    let quad = StandardGeometry::new_pair(quad_maker(), &renderer);
    let canvas = WGPUTexture::new_as_target(&renderer.device, (size.0 as u32, size.1 as u32));

    let mut pipeline_builder = WGPUPipelineDescriptorBuilder::new();
    pipeline_builder
      .vertex_shader(include_str!("./quad.vert"))
      .frag_shader(include_str!("./quad.frag"))
      .binding_group(
        BindGroupLayoutBuilder::new()
          .bind_uniform_buffer(ShaderStage::Vertex)
          .bind_texture2d(ShaderStage::Fragment)
          .bind_sampler(ShaderStage::Fragment),
      )
      .with_color_target(&canvas);
      // .with_swapchain_target(&renderer.swap_chain.swap_chain_descriptor);

    let quad_pipeline = pipeline_builder.build::<StandardGeometry>(&renderer.device);

    let camera = OrthographicCamera::new();
    let mx_total = OPENGL_TO_WGPU_MATRIX * camera.get_vp_matrix();
    let mx_ref: &[f32; 16] = mx_total.as_ref();
    let camera_gpu_buffer = WGPUBuffer::new(
      &renderer.device,
      mx_ref,
      wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
    );

    GUIRenderer {
      quad,
      view: Vec4::new(0.0, 0.0, size.0, size.1),
      camera,
      camera_gpu_buffer,
      canvas,
      quad_pipeline,
    }
  }

  pub fn draw_rect(
    &mut self,
    renderer: &mut WGPURenderer,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
  ) {
    let bindgroup = BindGroupBuilder::new()
    .buffer(&self.camera_gpu_buffer)
    .build(&renderer.device, &self.quad_pipeline.bind_group_layouts[0]);

    let mut pass = WGPURenderPass::build()
      .output(self.canvas.view())
      .create(&mut renderer.encoder);

    pass.gpu_pass.set_pipeline(&self.quad_pipeline.pipeline);
    pass
      .gpu_pass
      .set_bind_group(0, &bindgroup.gpu_bindgroup, &[]);
    
  }
}
