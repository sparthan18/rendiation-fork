use crate::*;

#[repr(C)]
#[std140_layout]
#[derive(Copy, Clone, ShaderStruct, Default)]
pub struct PointLightShaderInfo {
  pub luminance_intensity: Vec3<f32>,
  pub position: Vec3<f32>,
  pub cutoff_distance: f32,
}

impl PunctualShaderLight for PointLightShaderInfo {
  type PunctualDependency = ();

  fn create_punctual_dep(
    _: &mut ShaderGraphFragmentBuilderView,
  ) -> Result<Self::PunctualDependency, ShaderGraphBuildError> {
    Ok(())
  }

  fn compute_incident_light(
    _: &ShaderGraphFragmentBuilderView,
    light: &ENode<Self>,
    _dep: &Self::PunctualDependency,
    ctx: &ENode<ShaderLightingGeometricCtx>,
  ) -> Result<ENode<ShaderIncidentLight>, ShaderGraphBuildError> {
    let direction = ctx.position - light.position;
    let distance = direction.length();
    let factor = punctual_light_intensity_to_illuminance_factor(distance, light.cutoff_distance);

    Ok(ENode::<ShaderIncidentLight> {
      color: light.luminance_intensity * factor,
      direction: direction.normalize(),
    })
  }
}

impl WebGPULight for SceneItemRef<PointLight> {
  type Uniform = PointLightShaderInfo;

  fn create_uniform_stream(
    &self,
    ctx: &mut LightResourceCtx,
    node: Box<dyn Stream<Item = SceneNode>>,
  ) -> impl Stream<Item = Self::Uniform> {
    enum ShaderInfoDelta {
      Position(Vec3<f32>),
      // Shadow(LightShadowAddressInfo),
      Light(Vec3<f32>, f32),
    }

    let direction = node
      .map(|node| ctx.derives.create_world_matrix_stream(&node))
      .flatten_signal()
      .map(|mat| mat.position())
      .map(ShaderInfoDelta::Position);

    let ill = self
      .single_listen_by(any_change)
      .filter_map_sync(self.defer_weak())
      .map(|light| {
        (
          light.illuminance * light.color_factor,
          light.cutoff_distance,
        )
      })
      .map(ShaderInfoDelta::Light);

    let delta = futures::stream_select!(direction, ill);

    delta.fold_signal(DirectionalLightShaderInfo::default(), |delta, info| {
      match delta {
        ShaderInfoDelta::Position(position) => info.position = position,
        ShaderInfoDelta::Light(i, cutoff_distance) => {
          info.illuminance = i;
          info.cutoff_distance = cutoff_distance;
        }
      };
      Some(())
    })
  }
}

wgsl_fn!(
  // based upon Frostbite 3 Moving to Physically-based Rendering
  // page 32, equation 26: E[window1]
  // https://seblagarde.files.wordpress.com/2015/07/course_notes_moving_frostbite_to_pbr_v32.pdf
  // this is intended to be used on spot and point lights who are represented as luminous intensity
  // but who must be converted to illuminance for surface lighting calculation
  fn punctual_light_intensity_to_illuminance_factor(
    light_distance: f32,
    cutoff_distance: f32,
  ) -> f32 {
    let distance_falloff = 1.0 / max(pow(light_distance, 2.), 0.01);

    // should I use pow2 pow4 for optimization?

    //  todo use saturate (naga issue)
    let cutoff = pow(
      clamp(1.0 - pow(light_distance / cutoff_distance, 4.), 0., 1.),
      2.,
    );

    return distance_falloff * cutoff;
  }
);
