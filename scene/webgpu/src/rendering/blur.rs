use crate::*;

#[repr(C)]
#[std140_layout]
#[derive(Clone, Copy, ShaderStruct)]
pub struct LinearBlurConfig {
  pub direction: Vec2<f32>,
}

/// we separate this struct because weights data is decoupled with the blur direction
#[repr(C)]
#[std140_layout]
#[derive(Clone, Copy, ShaderStruct)]
pub struct ShaderSamplingWeights {
  /// we max support 32 weight, but maybe not used them all.
  /// this array is just used as a fixed size container.
  pub weights: Shader140Array<f32, 32>,
  /// the actually sample count we used.
  pub weight_count: u32,
}

pub struct LinearBlurTask<'a, T> {
  input: AttachmentView<T>,
  config: &'a UniformBufferDataView<LinearBlurConfig>,
  weights: &'a UniformBufferDataView<ShaderSamplingWeights>,
}

impl<'a, T> ShaderHashProvider for LinearBlurTask<'a, T> {}
impl<'a, T> ShaderGraphProvider for LinearBlurTask<'a, T> {
  fn build(
    &self,
    builder: &mut ShaderGraphRenderPipelineBuilder,
  ) -> Result<(), ShaderGraphBuildError> {
    builder.fragment(|builder, binding| {
      let config = binding.uniform_by(self.config, SB::Material).expand();
      let weights = binding.uniform_by(self.weights, SB::Material);

      let input = binding.uniform_by(&self.input, SB::Material);
      let sampler = binding.uniform::<GPUSamplerView>(SB::Material);

      let uv = builder.query::<FragmentUv>()?.get();
      let size = builder.query::<TexelSize>()?.get();

      let blurred = linear_blur(config.direction, weights, input, sampler, uv, size);

      builder.set_fragment_out(0, blurred)
    })
  }
}
impl<'a, T> ShaderPassBuilder for LinearBlurTask<'a, T> {
  fn setup_pass(&self, ctx: &mut GPURenderPassCtx) {
    ctx.binding.bind(self.config, SB::Material);
    ctx.binding.bind(self.weights, SB::Material);
    ctx.binding.bind(&self.input, SB::Material);
    ctx.bind_immediate_sampler(&TextureSampler::default(), SB::Material);
  }
}

pub fn draw_cross_blur<T>(
  config: &CrossBlurData,
  input: AttachmentView<T>,
  ctx: &mut FrameCtx,
) -> Attachment {
  let x_result = draw_linear_blur(&config.x, &config.weights, input, ctx);
  let y_result = draw_linear_blur(&config.y, &config.weights, x_result.read(), ctx);
  y_result
}

pub struct CrossBlurData {
  x: UniformBufferDataView<LinearBlurConfig>,
  y: UniformBufferDataView<LinearBlurConfig>,
  weights: UniformBufferDataView<ShaderSamplingWeights>,
}

pub fn draw_linear_blur<T>(
  config: &UniformBufferDataView<LinearBlurConfig>,
  weights: &UniformBufferDataView<ShaderSamplingWeights>,
  input: AttachmentView<T>,
  ctx: &mut FrameCtx,
) -> Attachment {
  let attachment: &Attachment = todo!();
  let des = attachment.des().clone();
  let dst = des.request(ctx);

  let task = LinearBlurTask {
    input,
    config,
    weights,
  };

  pass("blur")
    .with_color(dst.write(), load())
    .render(ctx)
    .by(task.draw_quad());

  dst
}

wgsl_fn!(
  fn lin_space(w0: f32, d0: vec4<f32>, w1: f32, d1: vec4<f32>) -> f32 {
    return (w0 * d0 + w1 * d1);
  }
);

wgsl_fn!(
  fn linear_blur(
    direction: vec2<f32>,
    weights: ShaderSamplingWeights,
    texture: texture_2d<f32>,
    sp: sampler,
    uv: vec2<f32>,
    texel_size: vec2<f32>,
  ) -> vec4<f32> {
    let sample_offset = texel_size * direction;
    var sum: vec4<f32>;
    for (var i: i32 = 2; i < weights.weight_count; i++) {
        let samples = textureSample(texture, sp, uv + f32(i) * sample_offset);
        sum = lin_space(1.0, sum, weights.weights[i], samples);
    }
    return sum;
  }
);
