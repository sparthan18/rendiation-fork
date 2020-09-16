use crate::WGPURenderer;

pub trait VertexProvider {
  fn get_buffer_layout_descriptor() -> wgpu::VertexBufferDescriptor<'static>;
}
pub trait GeometryProvider {
  fn get_geometry_vertex_state_descriptor() -> wgpu::VertexStateDescriptor<'static>;
  fn get_primitive_topology() -> wgpu::PrimitiveTopology;
}

pub trait BindGroupLayoutProvider: Sized + 'static {
  fn provide_layout(renderer: &WGPURenderer) -> wgpu::BindGroupLayout;
}