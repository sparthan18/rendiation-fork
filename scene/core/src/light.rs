use crate::*;

#[non_exhaustive]
#[derive(Clone)]
pub enum SceneLightKind {
  PointLight(SceneItemRef<PointLight>),
  SpotLight(SceneItemRef<SpotLight>),
  DirectionalLight(SceneItemRef<DirectionalLight>),
  Foreign(Box<dyn AnyClone + Send + Sync>),
}

clone_self_incremental!(SceneLightKind);

pub type SceneLight = SceneItemRef<SceneLightInner>;

#[derive(Incremental)]
pub struct SceneLightInner {
  pub light: SceneLightKind,
  /// Note: Light properties are unaffected by node transforms by default
  /// — for example, range and intensity do not change with scale.
  pub node: SceneNode,
}

#[derive(Debug, Clone, Incremental)]
pub struct PointLight {
  pub color_factor: Vec3<f32>,
  /// in cd
  pub luminance_intensity: f32,
  /// in meter
  pub cutoff_distance: f32,
  pub ext: DynamicExtension,
}

impl PointLight {
  /// The luminous power of a point light is calculated by integrating
  /// the luminous intensity over the light's solid angle
  ///
  /// return in watt
  pub fn compute_luminous_power(&self) -> f32 {
    f32::PI() * 4. * self.luminance_intensity
  }
  // in watts
  pub fn set_luminous_intensity_by_luminous_power(&mut self, luminous_power: f32) -> &mut Self {
    self.luminance_intensity = luminous_power / f32::PI() * 4.;
    self
  }
}

#[derive(Debug, Clone, Incremental)]
pub struct SpotLight {
  pub color_factor: Vec3<f32>,
  /// in cd
  pub luminance_intensity: f32,
  /// in meter
  pub cutoff_distance: f32,
  pub half_cone_angle: f32,
  /// should less equal to half_cont_angle,large equal to zero
  pub half_penumbra_angle: f32,
  pub ext: DynamicExtension,
}

impl SpotLight {
  /// luminous power of a spot light can be calculated in a similar fashion to
  /// point lights, using θ outer the outer angle of the spot light's cone in the range [0..π].
  ///
  /// return in watt
  pub fn compute_luminous_power(&self) -> f32 {
    f32::PI() * 2. * (1. - self.half_cone_angle.cos()) * self.luminance_intensity
  }
  // in watts
  pub fn set_luminous_intensity_by_luminous_power(&mut self, luminous_power: f32) -> &mut Self {
    self.luminance_intensity =
      luminous_power / (f32::PI() * 2. * (1. - self.half_cone_angle.cos()));
    self
  }
}

#[derive(Debug, Clone, Incremental)]
pub struct DirectionalLight {
  /// in lux
  ///
  /// for reference, the sun is 90000 ~ 130000 lux
  pub illuminance: f32,
  pub color_factor: Vec3<f32>,
  pub ext: DynamicExtension,
}
