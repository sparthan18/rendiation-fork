use rendiation_algebra::SquareMatrix;

use crate::*;

pub fn make_builtin_call<T: ShaderGraphNodeType>(
  ty: ShaderBuiltInFunction,
  params: impl IntoIterator<Item = ShaderGraphNodeRawHandle>,
) -> Node<T> {
  ShaderGraphNodeExpr::FunctionCall {
    meta: ShaderFunctionType::BuiltIn(ty),
    parameters: params.into_iter().collect(),
  }
  .insert_graph()
}

impl<T> Node<T>
where
  T: InnerProductSpace<f32> + PrimitiveShaderGraphNodeType,
{
  pub fn normalize(self) -> Self {
    make_builtin_call(ShaderBuiltInFunction::Normalize, [self.handle()])
  }

  pub fn length(self) -> Node<f32> {
    make_builtin_call(ShaderBuiltInFunction::Length, [self.handle()])
  }
}

impl<T> Node<T>
where
  T: SquareMatrix<f32> + PrimitiveShaderGraphNodeType,
{
  pub fn inverse(self) -> Self {
    make_builtin_call(ShaderBuiltInFunction::MatInverse, [self.handle()])
  }
  pub fn transpose(self) -> Self {
    make_builtin_call(ShaderBuiltInFunction::MatTranspose, [self.handle()])
  }
}

impl Node<Mat4<f32>> {
  pub fn position(self) -> Node<Vec3<f32>> {
    self.nth_colum(3).xyz()
  }
  pub fn nth_colum(self, n: impl Into<Node<i32>>) -> Node<Vec4<f32>> {
    ShaderGraphNodeExpr::Operator(OperatorNode::Index {
      array: self.handle(),
      entry: n.into().handle(),
    })
    .insert_graph()
  }
}
