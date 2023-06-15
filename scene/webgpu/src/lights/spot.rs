use crate::*;

#[repr(C)]
#[std140_layout]
#[derive(Copy, Clone, ShaderStruct, Default)]
pub struct SpotLightShaderInfo {
  pub luminance_intensity: Vec3<f32>,
  pub position: Vec3<f32>,
  pub direction: Vec3<f32>,
  pub cutoff_distance: f32,
  pub half_cone_cos: f32,
  pub half_penumbra_cos: f32,
  pub shadow: LightShadowAddressInfo,
}

impl PunctualShaderLight for SpotLightShaderInfo {
  type PunctualDependency = ();

  fn create_punctual_dep(
    _: &mut ShaderGraphFragmentBuilderView,
  ) -> Result<Self::PunctualDependency, ShaderGraphBuildError> {
    Ok(())
  }

  fn compute_incident_light(
    builder: &ShaderGraphFragmentBuilderView,
    light: &ENode<Self>,
    _dep: &Self::PunctualDependency,
    ctx: &ENode<ShaderLightingGeometricCtx>,
  ) -> Result<ENode<ShaderIncidentLight>, ShaderGraphBuildError> {
    let direction = ctx.position - light.position;
    let distance = direction.length();
    let distance_factor =
      punctual_light_intensity_to_illuminance_factor(distance, light.cutoff_distance);

    let direction = direction.normalize();
    let angle_cos = direction.dot(light.direction);
    let angle_factor = angle_cos.smoothstep(light.half_cone_cos, light.half_penumbra_cos);

    let shadow_info = light.shadow.expand();
    let occlusion = consts(1.).mutable();

    let intensity_factor = distance_factor * angle_factor;

    if_by_ok(shadow_info.enabled.equals(consts(1)), || {
      let map = builder.query::<BasicShadowMap>().unwrap();
      let sampler = builder.query::<BasicShadowMapSampler>().unwrap();

      let shadow_infos = builder.query::<BasicShadowMapInfoGroup>().unwrap();
      let shadow_info = shadow_infos.index(shadow_info.index).expand();

      let shadow_position = compute_shadow_position(builder, shadow_info)?;

      // we should have kept all light effective places inside the shadow volume
      if_by(intensity_factor.greater_than(consts(0.)), || {
        occlusion.set(sample_shadow(
          shadow_position,
          map,
          sampler,
          shadow_info.map_info,
        ))
      });
      Ok(())
    })?;

    let shadow_factor = consts(1.) - occlusion.get();

    Ok(ENode::<ShaderIncidentLight> {
      color: light.luminance_intensity * intensity_factor * shadow_factor,
      direction,
    })
  }
}

impl WebGPULight for SceneItemRef<SpotLight> {
  type Uniform = SpotLightShaderInfo;

  fn create_uniform_stream(
    &self,
    ctx: &mut LightResourceCtx,
    node: Box<dyn Stream<Item = SceneNode>>,
  ) -> impl Stream<Item = Self::Uniform> {
    enum ShaderInfoDelta {
      DirPosition(Vec3<f32>, Vec3<f32>),
      Shadow(LightShadowAddressInfo),
      Light(SpotLightShaderInfoPart),
    }

    struct SpotLightShaderInfoPart {
      pub luminance_intensity: Vec3<f32>,
      pub cutoff_distance: f32,
      pub half_cone_cos: f32,
      pub half_penumbra_cos: f32,
    }

    let direction = node
      .map(|node| ctx.derives.create_world_matrix_stream(&node))
      .flatten_signal()
      .map(|mat| (mat.forward().reverse().normalize(), mat.position()))
      .map(ShaderInfoDelta::DirPosition);

    let shadow = ctx
      .shadow_system()
      .create_basic_shadow_stream(&self)
      .map(ShaderInfoDelta::Shadow);

    let light = self
      .single_listen_by(any_change)
      .filter_map_sync(self.defer_weak())
      .map(|light| SpotLightShaderInfoPart {
        luminance_intensity: light.luminance_intensity * light.color_factor,
        cutoff_distance: light.cutoff_distance,
        half_cone_cos: light.half_cone_angle.cos(),
        half_penumbra_cos: light.half_penumbra_angle.cos(),
      })
      .map(ShaderInfoDelta::Light);

    let delta = futures::stream_select!(direction, shadow, light);

    delta.fold_signal(SpotLightShaderInfo::default(), |delta, info| {
      match delta {
        ShaderInfoDelta::DirPosition((dir, pos)) => {
          info.direction = dir;
          info.position = pos;
        }
        ShaderInfoDelta::Shadow(shadow) => info.shadow = shadow,
        ShaderInfoDelta::Light(l) => {
          info.luminance_intensity = l.luminance_intensity;
          info.cutoff_distance = l.cutoff_distance;
          info.half_penumbra_cos = l.half_penumbra_cos;
          info.half_cone_cos = l.half_cone_cos;
        }
      };
      Some(())
    })
  }
}

impl ShadowSingleProjectCreator for SceneItemRef<SpotLight> {
  fn build_shadow_projection(&self) -> Option<impl Stream<Item = Box<dyn CameraProjection>>> {
    let proj = PerspectiveProjection {
      near: 0.1,
      far: 2000.,
      fov: Deg::from_rad(self.read().half_cone_angle * 2.),
      aspect: 1.,
    };
    let proj = CameraProjector::Perspective(proj);
    SceneCamera::create(proj, node.clone())
  }
}
