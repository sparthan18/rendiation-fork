use crate::{
  modify_graph, AnyType, ShaderGraphAttributeNodeType, ShaderGraphBindGroupBuilder,
  ShaderGraphBindGroupItemProvider, ShaderGraphConstableNodeType, ShaderGraphNode,
  ShaderGraphNodeData, ShaderGraphNodeHandle, ShaderGraphNodeType, TextureSamplingNode,
};
use rendiation_math::*;
use rendiation_ral::ShaderStage;

impl ShaderGraphNodeType for AnyType {
  fn to_glsl_type() -> &'static str {
    unreachable!("Node can't newed with type AnyType")
  }
}

impl ShaderGraphNodeType for f32 {
  fn to_glsl_type() -> &'static str {
    "float"
  }
}
impl ShaderGraphAttributeNodeType for f32 {}
impl ShaderGraphConstableNodeType for f32 {
  fn const_to_glsl(&self) -> String {
    let mut result = format!("{}", self);
    if result.contains(".") {
      result
    } else {
      result.push_str(".0");
      result
    }
  }
}

impl ShaderGraphNodeType for Vec2<f32> {
  fn to_glsl_type() -> &'static str {
    "vec2"
  }
}
impl ShaderGraphAttributeNodeType for Vec2<f32> {}
impl ShaderGraphConstableNodeType for Vec2<f32> {
  fn const_to_glsl(&self) -> String {
    format!(
      "vec2({}, {})",
      self.x.const_to_glsl(),
      self.y.const_to_glsl()
    )
  }
}

impl ShaderGraphNodeType for Vec3<f32> {
  fn to_glsl_type() -> &'static str {
    "vec3"
  }
}
impl ShaderGraphAttributeNodeType for Vec3<f32> {}
impl ShaderGraphConstableNodeType for Vec3<f32> {
  fn const_to_glsl(&self) -> String {
    format!(
      "vec3({}, {}, {}",
      self.x.const_to_glsl(),
      self.y.const_to_glsl(),
      self.z.const_to_glsl()
    )
  }
}

impl ShaderGraphNodeType for Vec4<f32> {
  fn to_glsl_type() -> &'static str {
    "vec4"
  }
}
impl ShaderGraphAttributeNodeType for Vec4<f32> {}
impl ShaderGraphConstableNodeType for Vec4<f32> {
  fn const_to_glsl(&self) -> String {
    format!(
      "vec4({}, {}, {}, {}",
      self.x.const_to_glsl(),
      self.y.const_to_glsl(),
      self.z.const_to_glsl(),
      self.w.const_to_glsl()
    )
  }
}

impl ShaderGraphNodeType for Mat4<f32> {
  fn to_glsl_type() -> &'static str {
    "mat4"
  }
}

#[derive(Copy, Clone)]
pub struct ShaderGraphSampler;

impl ShaderGraphNodeType for ShaderGraphSampler {
  fn to_glsl_type() -> &'static str {
    "sampler"
  }
}

impl ShaderGraphBindGroupItemProvider for ShaderGraphSampler {
  type ShaderGraphBindGroupItemInstance = ShaderGraphNodeHandle<ShaderGraphSampler>;

  fn create_instance<'a>(
    name: &'static str,
    bindgroup_builder: &mut ShaderGraphBindGroupBuilder<'a>,
    stage: ShaderStage,
  ) -> Self::ShaderGraphBindGroupItemInstance {
    let node = bindgroup_builder.create_uniform_node::<ShaderGraphSampler>(name);
    bindgroup_builder.add_none_ubo(unsafe { node.handle.cast_type().into() }, stage);
    node
  }
}

#[derive(Copy, Clone)]
pub struct ShaderGraphTexture;

impl ShaderGraphNodeType for ShaderGraphTexture {
  fn to_glsl_type() -> &'static str {
    "texture2D"
  }
}

impl ShaderGraphNodeHandle<ShaderGraphTexture> {
  pub fn sample(
    &self,
    sampler: ShaderGraphNodeHandle<ShaderGraphSampler>,
    position: ShaderGraphNodeHandle<Vec2<f32>>,
  ) -> ShaderGraphNodeHandle<Vec4<f32>> {
    modify_graph(|g| {
      let node = ShaderGraphNode::<Vec4<f32>>::new(ShaderGraphNodeData::TextureSampling(
        TextureSamplingNode {
          texture: self.handle,
          sampler: sampler.handle,
          position: position.handle,
        },
      ));
      let handle = g.nodes.create_node(node.to_any());
      unsafe {
        g.nodes.connect_node(sampler.handle.cast_type(), handle);
        g.nodes.connect_node(position.handle.cast_type(), handle);
        g.nodes.connect_node(self.handle.cast_type(), handle);
        handle.cast_type().into()
      }
    })
  }
}

impl ShaderGraphBindGroupItemProvider for ShaderGraphTexture {
  type ShaderGraphBindGroupItemInstance = ShaderGraphNodeHandle<ShaderGraphTexture>;

  fn create_instance<'a>(
    name: &'static str,
    bindgroup_builder: &mut ShaderGraphBindGroupBuilder<'a>,
    stage: ShaderStage,
  ) -> Self::ShaderGraphBindGroupItemInstance {
    let node = bindgroup_builder.create_uniform_node::<ShaderGraphTexture>(name);
    bindgroup_builder.add_none_ubo(unsafe { node.handle.cast_type().into() }, stage);
    node
  }
}