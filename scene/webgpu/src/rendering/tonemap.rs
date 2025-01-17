use crate::*;

pub struct ToneMap {
  ty: ToneMapType,
  exposure: UniformBufferDataView<f32>,
}

impl ToneMap {
  pub fn new(gpu: &GPU) -> Self {
    Self {
      ty: ToneMapType::Linear,
      exposure: create_uniform(1., gpu),
    }
  }
}

impl ToneMap {
  pub fn tonemap<'a, T: 'a>(&'a self, hdr: AttachmentView<T>) -> impl PassContent + 'a {
    ToneMapTask { hdr, config: self }.draw_quad()
  }
}

pub enum ToneMapType {
  Linear,
  Reinhard,
  Cineon,
  ACESFilmic,
}
impl ShaderHashProvider for ToneMap {
  fn hash_pipeline(&self, hasher: &mut PipelineHasher) {
    std::mem::discriminant(&self.ty).hash(hasher)
  }
}
impl ShaderPassBuilder for ToneMap {
  fn setup_pass(&self, ctx: &mut GPURenderPassCtx) {
    ctx.binding.bind(&self.exposure);
  }
}
impl GraphicsShaderProvider for ToneMap {
  fn build(&self, builder: &mut ShaderRenderPipelineBuilder) -> Result<(), ShaderBuildError> {
    builder.fragment(|builder, binding| {
      let exposure = binding.bind_by(&self.exposure).load();
      let hdr = builder.query::<HDRLightResult>()?;

      let mapped = match self.ty {
        ToneMapType::Linear => linear_tone_mapping(hdr, exposure),
        ToneMapType::Reinhard => reinhard_tone_mapping(hdr, exposure),
        ToneMapType::Cineon => optimized_cineon_tone_mapping(hdr, exposure),
        ToneMapType::ACESFilmic => aces_filmic_tone_mapping(hdr, exposure),
      };

      builder.register::<LDRLightResult>(mapped);
      Ok(())
    })
  }
}

#[shader_fn]
fn linear_tone_mapping(color: Node<Vec3<f32>>, exposure: Node<f32>) -> Node<Vec3<f32>> {
  exposure * color
}

/// source: https://www.cs.utah.edu/docs/techreports/2002/pdf/UUCS-02-001.pdf
#[shader_fn]
fn reinhard_tone_mapping(color: Node<Vec3<f32>>, exposure: Node<f32>) -> Node<Vec3<f32>> {
  let color = exposure * color;
  let mapped = color / (val(Vec3::one()) + color);
  mapped.saturate()
}

// val vec3 splat
fn val_v3s(f: f32) -> Node<Vec3<f32>> {
  val(Vec3::splat(f))
}

/// optimized filmic operator by Jim Hejl and Richard Burgess-Dawson
/// source: http://filmicworlds.com/blog/filmic-tonemapping-operators/
fn optimized_cineon_tone_mapping(color: Node<Vec3<f32>>, exposure: Node<f32>) -> Node<Vec3<f32>> {
  let color = exposure * color;
  let color = (color - val_v3s(0.004)).max(Vec3::zero());
  let color = (color * (val(6.2) * color + val_v3s(0.5)))
    / (color * (val(6.2) * color + val_v3s(1.7)) + val_v3s(0.06));
  color.pow(2.2)
}

// source: https://github.com/selfshadow/ltc_code/blob/master/webgl/shaders/ltc/ltc_blit.fs
fn rrt_and_odt_fit(v: Node<Vec3<f32>>) -> Node<Vec3<f32>> {
  let a = v * (v + val_v3s(0.0245786)) - val_v3s(0.000090537);
  let b = v * (val(0.983729) * v + val_v3s(0.432951)) + val_v3s(0.238081);
  a / b
}

/// this implementation of ACES is modified to accommodate a brighter viewing environment.
/// the scale factor of 1/0.6 is subjective. see discussion in #19621 in three.js repo.
fn aces_filmic_tone_mapping(color: Node<Vec3<f32>>, exposure: Node<f32>) -> Node<Vec3<f32>> {
  // sRGB => XYZ => D65_2_D60 => AP1 => RRT_SAT
  let aces_input_mat: Node<Mat3<f32>> = (
    (val(0.59719), val(0.07600), val(0.02840)).into(), // transposed from source
    (val(0.35458), val(0.90834), val(0.13383)).into(),
    (val(0.04823), val(0.01566), val(0.83777)).into(),
  )
    .into();

  // ODT_SAT => XYZ => D60_2_D65 => sRGB
  let aces_output_mat: Node<Mat3<f32>> = (
    (val(1.60475), val(-0.10208), val(-0.00327)).into(), // transposed from source
    (val(-0.53108), val(1.10813), val(-0.07276)).into(),
    (val(-0.07367), val(-0.00605), val(1.07602)).into(),
  )
    .into();

  let mut color = color;
  color *= (exposure / val(0.6)).splat();

  color = aces_input_mat * color;

  // Apply RRT and ODT
  color = rrt_and_odt_fit(color);

  color = aces_output_mat * color;

  color.saturate()
}

struct ToneMapTask<'a, T> {
  hdr: AttachmentView<T>,
  config: &'a ToneMap,
}

impl<'a, T> ShaderHashProviderAny for ToneMapTask<'a, T> {
  fn hash_pipeline_and_with_type_id(&self, hasher: &mut PipelineHasher) {
    self.config.type_id().hash(hasher);
    self.hash_pipeline(hasher);
  }
}
impl<'a, T> ShaderHashProvider for ToneMapTask<'a, T> {
  fn hash_pipeline(&self, hasher: &mut PipelineHasher) {
    self.config.hash_pipeline(hasher)
  }
}
impl<'a, T> ShaderPassBuilder for ToneMapTask<'a, T> {
  fn setup_pass(&self, ctx: &mut GPURenderPassCtx) {
    ctx.binding.bind(&self.hdr);
    ctx.bind_immediate_sampler(&TextureSampler::default().into_gpu());
    self.config.setup_pass(ctx)
  }
}

impl<'a, T> GraphicsShaderProvider for ToneMapTask<'a, T> {
  fn build(&self, builder: &mut ShaderRenderPipelineBuilder) -> Result<(), ShaderBuildError> {
    builder.fragment(|builder, binding| {
      let hdr = binding.bind_by_unchecked(&self.hdr);
      let sampler = binding.binding::<GPUSamplerView>();

      let uv = builder.query::<FragmentUv>()?;
      let hdr = hdr.sample(sampler, uv).xyz();

      builder.register::<HDRLightResult>(hdr);
      Ok(())
    })?;

    self.config.build(builder)?;

    builder.fragment(|builder, _| {
      let ldr = builder.query::<LDRLightResult>()?;
      builder.store_fragment_out(0, (ldr, val(1.)))
    })
  }
}
