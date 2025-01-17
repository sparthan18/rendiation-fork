pub mod physical;
pub use physical::*;

use crate::*;

pub trait LightableSurfaceShading: Any {
  type ShaderStruct: ShaderStructuralNodeType;
  /// define how we construct a shader material instance from shader build ctx
  fn construct_shading(builder: &mut ShaderFragmentBuilder) -> ENode<Self::ShaderStruct>;

  /// define how we compute result lighting from a give pixel of surface and lighting
  fn compute_lighting_by_incident(
    self_node: &ENode<Self::ShaderStruct>,
    incident: &ENode<ShaderIncidentLight>,
    ctx: &ENode<ShaderLightingGeometricCtx>,
  ) -> Result<ENode<ShaderLightingResult>, ShaderBuildError>;
}

pub trait LightableSurfaceShadingDyn: Any {
  fn construct_shading_dyn(&self, builder: &mut ShaderFragmentBuilder) -> Box<dyn Any>;

  fn compute_lighting_by_incident_dyn(
    &self,
    self_node: &dyn Any,
    direct_light: &ENode<ShaderIncidentLight>,
    ctx: &ENode<ShaderLightingGeometricCtx>,
  ) -> Result<ENode<ShaderLightingResult>, ShaderBuildError>;
}
impl<T: LightableSurfaceShading> LightableSurfaceShadingDyn for T {
  fn construct_shading_dyn(&self, builder: &mut ShaderFragmentBuilder) -> Box<dyn Any> {
    Box::new(Self::construct_shading(builder))
  }

  fn compute_lighting_by_incident_dyn(
    &self,
    self_node: &dyn Any,
    direct_light: &ENode<ShaderIncidentLight>,
    ctx: &ENode<ShaderLightingGeometricCtx>,
  ) -> Result<ENode<ShaderLightingResult>, ShaderBuildError> {
    let self_node = self_node
      .downcast_ref::<ENode<<Self as LightableSurfaceShading>::ShaderStruct>>()
      .unwrap();
    Self::compute_lighting_by_incident(self_node, direct_light, ctx)
  }
}
