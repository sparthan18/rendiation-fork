use std::borrow::Cow;

use rendiation_algebra::Vec3;
use rendiation_algebra::Vector;
use rendiation_renderable_mesh::tessellation::IndexedMeshTessellator;
use rendiation_renderable_mesh::tessellation::SphereMeshParameter;
use rendiation_renderable_mesh::vertex::Vertex;
use rendiation_renderable_mesh::MeshGPU;
use rendiation_webgpu::*;

use crate::CameraBindgroup;
use crate::PipelineCreateCtx;
use crate::PreferredMaterialStates;
use crate::TextureCubeHandle;
use crate::TransformGPU;

pub trait Background: 'static + Renderable {
  fn require_pass_clear(&self) -> Option<wgpu::Color>;
}

pub struct SolidBackground {
  pub intensity: Vec3<f32>,
}

impl Renderable for SolidBackground {
  fn update(&mut self, _: &GPU, _: &mut wgpu::CommandEncoder) {}

  fn setup_pass<'a>(&'a self, _: &mut wgpu::RenderPass<'a>) {}
}

impl Background for SolidBackground {
  fn require_pass_clear(&self) -> Option<wgpu::Color> {
    wgpu::Color {
      r: self.intensity.r() as f64,
      g: self.intensity.g() as f64,
      b: self.intensity.b() as f64,
      a: 1.,
    }
    .into()
  }
}

impl Default for SolidBackground {
  fn default() -> Self {
    Self {
      intensity: Vec3::new(0.6, 0.6, 0.6),
    }
  }
}

impl SolidBackground {
  pub fn black() -> Self {
    Self {
      intensity: Vec3::splat(0.0),
    }
  }
}

pub struct DrawableBackground<S> {
  mesh: MeshGPU,
  pub shading: S,
}

impl<S> Renderable for DrawableBackground<S> {
  fn update(&mut self, _: &GPU, _: &mut wgpu::CommandEncoder) {}

  fn setup_pass<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
    self.mesh.setup_pass(pass, None);
  }
}

impl<S> DrawableBackground<S> {
  pub fn new(device: &wgpu::Device, camera: &CameraBindgroup, shading: S) -> Self {
    let mesh = SphereMeshParameter::default().tessellate();
    let mesh = mesh.mesh.create_gpu(device);

    Self { mesh, shading }
  }
}

pub struct EnvMapGround {
  pub texture: TextureCubeHandle,
}

pub trait BackGroundShading {
  fn shader_header(&self) -> &'static str;

  fn shading(&self) -> &'static str;

  fn shader(&self) -> String {
    format!(
      "
    {object_header}
    {material_header}
    {camera_header}

    {background_shading}

    struct VertexOutput {{
      [[builtin(position)]] position: vec4<f32>;
      [[location(0)]] uv: vec2<f32>;
    }};

    [[stage(vertex)]]
    fn vs_main(
      {vertex_header}
    ) -> VertexOutput {{
      var out: VertexOutput;
      out.uv = uv;
      out.position = camera.projection * camera.view * model.matrix * vec4<f32>(position, 1.0);
      out.position.w = 1.0;
      return out;
    }}
    
    [[stage(fragment)]]
    fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {{
      let direction = normalize(in.position);
      return background_shading(direction);
    }}
    ",
      vertex_header = Vertex::get_shader_header(),
      material_header = self.shader_header(),
      camera_header = CameraBindgroup::get_shader_header(),
      object_header = TransformGPU::get_shader_header(),
      background_shading = self.shading()
    )
  }

  fn create_bindgroup_layout(&self, device: &wgpu::Device) -> wgpu::BindGroupLayout;

  fn create_pipeline(
    &self,
    device: &wgpu::Device,
    ctx: &PipelineCreateCtx,
  ) -> wgpu::RenderPipeline {
    let bindgroup_layout = self.create_bindgroup_layout(device);
    let shader_source = self.shader();

    let states = PreferredMaterialStates {
      depth_write_enabled: false,
      ..Default::default()
    };

    let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
      label: None,
      source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(shader_source.as_str())),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: None,
      bind_group_layouts: &[
        &ctx.model_gpu.layout,
        &bindgroup_layout,
        &ctx.camera_gpu.layout,
      ],
      push_constant_ranges: &[],
    });

    let vertex_buffers = vec![Vertex::vertex_layout()];

    let targets: Vec<_> = ctx
      .pass
      .color_format()
      .iter()
      .map(|&f| states.map_color_states(f))
      .collect();

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: None,
      layout: Some(&pipeline_layout),
      vertex: wgpu::VertexState {
        module: &shader,
        entry_point: "vs_main",
        buffers: &vertex_buffers,
      },
      fragment: Some(wgpu::FragmentState {
        module: &shader,
        entry_point: "fs_main",
        targets: targets.as_slice(),
      }),
      primitive: wgpu::PrimitiveState {
        cull_mode: None,
        topology: wgpu::PrimitiveTopology::TriangleList,
        ..Default::default()
      },
      depth_stencil: states.map_depth_stencil_state(ctx.pass.depth_stencil_format()),
      multisample: wgpu::MultisampleState::default(),
    })
  }
}

impl BackGroundShading for EnvMapGround {
  fn shading(&self) -> &'static str {
    "
    fn background_shading(direction: vec3<f32>) -> vec3<f32> {
      textureSample(r_color, r_sampler, direction);
    }
    "
  }

  fn shader_header(&self) -> &'static str {
    "
    [[group(1), binding(0)]]
    var r_color: texture_cube<f32>;

    [[group(1), binding(1)]]
    var r_sampler: sampler;
    "
  }

  fn create_bindgroup_layout(&self, device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: None,
      entries: &[
        wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: WebGPUTextureCube::bind_layout(),
          count: None,
        },
        wgpu::BindGroupLayoutEntry {
          binding: 1,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::Sampler::bind_layout(),
          count: None,
        },
      ],
    })
  }
}
