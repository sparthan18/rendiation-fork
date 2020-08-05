use crate::{ShaderGraphBuilder, ShaderGraphBindGroupBuilder};

pub trait ShaderGraphUniformBufferProvider {
  type ShaderGraphUniformBufferInstance;

  fn create_instance<'a>(
    bindgroup_builder: &mut ShaderGraphBindGroupBuilder<'a>,
  ) -> Self::ShaderGraphUniformBufferInstance;
}

pub trait ShaderGraphBindGroupProvider{
    type ShaderGraphBindGroupInstance;

    fn create_instance<'a>(
        bindgroup_builder: &mut ShaderGraphBuilder<'a>,
    ) -> Self::ShaderGraphBindGroupInstance;

}

// pub trait ShaderGraphUniformBuffer {
//   type ShaderGraphUniformBufferInstance;

//   fn create_instance<'a>(
//     bindgroup_builder: &mut ShaderGraphBindGroupBuilder<'a>,
//   ) -> Self::ShaderGraphUniformBufferInstance;
// }
