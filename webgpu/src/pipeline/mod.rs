use crate::*;

use shadergraph::*;
use wgsl_codegen_graph::*;
pub mod container;
pub use container as c;

#[derive(Clone)]
pub struct GPURenderPipeline {
  pub inner: Rc<GPURenderPipelineInner>,
}

impl GPURenderPipeline {
  pub fn new(pipeline: gpu::RenderPipeline, bg_layouts: Vec<GPUBindGroupLayout>) -> Self {
    let inner = GPURenderPipelineInner {
      pipeline,
      bg_layouts,
    };
    Self {
      inner: Rc::new(inner),
    }
  }

  pub fn get_layout(&self, sb: SemanticBinding) -> &GPUBindGroupLayout {
    let index = sb.binding_index();
    self.bg_layouts.get(index).unwrap()
  }
}

pub struct GPURenderPipelineInner {
  pub pipeline: gpu::RenderPipeline,
  pub bg_layouts: Vec<GPUBindGroupLayout>,
}

impl Deref for GPURenderPipeline {
  type Target = GPURenderPipelineInner;

  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}

pub fn create_bindgroup_layout_by_node_ty<'a>(
  device: &GPUDevice,
  iter: impl Iterator<Item = (&'a ShaderValueType, gpu::ShaderStages)>,
) -> GPUBindGroupLayout {
  let entries: Vec<_> = iter
    .enumerate()
    .map(|(i, (ty, visibility))| {
      let ty = match ty {
        shadergraph::ShaderValueType::Fixed(_) => gpu::BindingType::Buffer {
          ty: gpu::BufferBindingType::Uniform,
          has_dynamic_offset: false,
          // min_binding_size: gpu::BufferSize::new(std::mem::size_of::<T>() as u64), // todo
          min_binding_size: None,
        },
        shadergraph::ShaderValueType::Sampler => {
          gpu::BindingType::Sampler(gpu::SamplerBindingType::Filtering)
        }
        shadergraph::ShaderValueType::Texture => gpu::BindingType::Texture {
          multisampled: false,
          sample_type: gpu::TextureSampleType::Float { filterable: true },
          view_dimension: gpu::TextureViewDimension::D2,
        },
        shadergraph::ShaderValueType::Never => unreachable!(),
        shadergraph::ShaderValueType::SamplerCombinedTexture => {
          todo!()
        }
      };

      gpu::BindGroupLayoutEntry {
        binding: i as u32,
        visibility,
        ty,
        count: None,
      }
    })
    .collect();

  device.create_and_cache_bindgroup_layout(entries.as_ref())
}

impl GPUDevice {
  pub fn build_pipeline_by_shadergraph(
    &self,
    builder: ShaderGraphRenderPipelineBuilder,
  ) -> Result<GPURenderPipeline, ShaderGraphBuildError> {
    let compile_result = builder.build(WGSL)?;

    let ShaderGraphCompileResult {
      shader,
      bindings,
      vertex_layouts,
      primitive_state,
      color_states,
      depth_stencil,
      multisample,
      target,
    } = compile_result;

    let WGSLShaderSource { vertex, fragment } = shader;
    println!("{}", vertex);
    println!("{}", fragment);

    let vertex = self.create_shader_module(&gpu::ShaderModuleDescriptor {
      label: None,
      source: gpu::ShaderSource::Wgsl(Cow::Borrowed(vertex.as_str())),
    });
    let fragment = self.create_shader_module(&gpu::ShaderModuleDescriptor {
      label: None,
      source: gpu::ShaderSource::Wgsl(Cow::Borrowed(fragment.as_str())),
    });

    let layouts: Vec<_> = bindings
      .bindings
      .iter()
      .map(|binding| {
        let iter = binding.bindings.iter().map(|(ty, vis)| {
          let visibility = match vis.get() {
            ShaderStageVisibility::Vertex => gpu::ShaderStages::VERTEX,
            ShaderStageVisibility::Fragment => gpu::ShaderStages::FRAGMENT,
            ShaderStageVisibility::Both => gpu::ShaderStages::VERTEX_FRAGMENT,
            ShaderStageVisibility::None => gpu::ShaderStages::NONE,
          };
          (ty, visibility)
        });

        create_bindgroup_layout_by_node_ty(self, iter)
      })
      .collect();

    let layouts_ref: Vec<_> = layouts.iter().map(|l| l.inner.as_ref()).collect();

    let pipeline_layout = self.create_pipeline_layout(&gpu::PipelineLayoutDescriptor {
      label: None,
      bind_group_layouts: layouts_ref.as_slice(),
      push_constant_ranges: &[],
    });

    let vertex_buffers: Vec<_> = vertex_layouts.iter().map(convert_vertex_layout).collect();

    let pipeline = self.create_render_pipeline(&gpu::RenderPipelineDescriptor {
      label: None,
      layout: Some(&pipeline_layout),
      vertex: gpu::VertexState {
        module: &vertex,
        entry_point: target.vertex_entry_name(),
        buffers: vertex_buffers.as_slice(),
      },
      fragment: Some(gpu::FragmentState {
        module: &fragment,
        entry_point: target.fragment_entry_name(),
        targets: color_states.as_slice(),
      }),
      primitive: primitive_state,
      depth_stencil,
      multisample,
      multiview: None,
    });

    Ok(GPURenderPipeline::new(pipeline, layouts))
  }
}

pub fn convert_vertex_layout(layout: &ShaderGraphVertexBufferLayout) -> gpu::VertexBufferLayout {
  gpu::VertexBufferLayout {
    array_stride: layout.array_stride,
    step_mode: layout.step_mode,
    attributes: layout.attributes.as_slice(),
  }
}
