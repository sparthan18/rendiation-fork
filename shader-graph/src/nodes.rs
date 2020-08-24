use crate::{ShaderFunction, ShaderGraphNodeUntyped};
use rendiation_math::*;
use std::{any::TypeId, marker::PhantomData, sync::Arc};

pub trait ShaderGraphNodeType: 'static {
  fn to_glsl_type() -> &'static str;
}

pub trait ShaderGraphConstableNodeType: 'static + Send + Sync {
  fn const_to_glsl(&self) -> String;
}

// this for not include samplers/textures as attributes
pub trait ShaderGraphAttributeNodeType: ShaderGraphNodeType {}

pub struct AnyType {}

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

impl ShaderGraphNodeType for Vec2<f32> {
  fn to_glsl_type() -> &'static str {
    "vec2"
  }
}
impl ShaderGraphAttributeNodeType for Vec2<f32> {}

impl ShaderGraphNodeType for Vec3<f32> {
  fn to_glsl_type() -> &'static str {
    "vec3"
  }
}
impl ShaderGraphAttributeNodeType for Vec3<f32> {}

impl ShaderGraphNodeType for Vec4<f32> {
  fn to_glsl_type() -> &'static str {
    "vec4"
  }
}
impl ShaderGraphAttributeNodeType for Vec4<f32> {}

impl ShaderGraphNodeType for Mat4<f32> {
  fn to_glsl_type() -> &'static str {
    "mat4"
  }
}

pub struct ShaderGraphNode<T: ShaderGraphNodeType> {
  phantom: PhantomData<T>,
  pub data: ShaderGraphNodeData,
  pub node_type: TypeId,
}

impl<T: ShaderGraphNodeType> ShaderGraphNode<T> {
  pub fn new(data: ShaderGraphNodeData) -> Self {
    Self {
      data,
      phantom: PhantomData,
      node_type: TypeId::of::<T>(),
    }
  }
  pub fn to_any(self) -> ShaderGraphNodeUntyped {
    unsafe { std::mem::transmute(self) }
  }
  pub fn from_any(self) -> ShaderGraphNode<T> {
    unsafe { std::mem::transmute(self) }
  }

  pub fn unwrap_as_input(&self) -> &ShaderGraphInputNode {
    match &self.data {
      ShaderGraphNodeData::Input(n) => n,
      _ => panic!("unwrap as input failed"),
    }
  }

  pub fn unwrap_as_vary(&self) -> usize {
    match &self.data {
      ShaderGraphNodeData::Output(ShaderGraphOutput::Vary(n)) => *n,
      _ => panic!("unwrap as input failed"),
    }
  }
}

pub enum ShaderGraphNodeData {
  Function(FunctionNode),
  Input(ShaderGraphInputNode),
  Output(ShaderGraphOutput),
  Const(Box<dyn ShaderGraphConstableNodeType>),
}

pub enum ShaderGraphOutput {
  Vary(usize),
  Frag(usize),
  Vert,
}

pub struct FunctionNode {
  pub prototype: Arc<ShaderFunction>,
}

pub struct ShaderGraphInputNode {
  pub node_type: ShaderGraphInputNodeType,
  pub name: String,
}

pub enum ShaderGraphInputNodeType {
  Uniform,
  Attribute,
  Vary,
}
