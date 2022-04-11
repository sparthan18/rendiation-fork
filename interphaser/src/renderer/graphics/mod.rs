use rendiation_algebra::*;
use shadergraph::*;

#[derive(Debug, Copy, Clone, ShaderVertex)]
pub struct UIVertex {
  #[semantic(GeometryPosition)]
  position: Vec2<f32>,
  #[semantic(GeometryUV)]
  uv: Vec2<f32>,
  #[semantic(GeometryColor)]
  color: Vec4<f32>,
}
unsafe impl bytemuck::Zeroable for UIVertex {}
unsafe impl bytemuck::Pod for UIVertex {}

pub fn vertex(position: (f32, f32), uv: (f32, f32), color: (f32, f32, f32, f32)) -> UIVertex {
  UIVertex {
    position: position.into(),
    uv: uv.into(),
    color: color.into(),
  }
}