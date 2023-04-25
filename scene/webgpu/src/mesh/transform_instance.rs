use crate::*;

pub struct TransformInstanceGPU {
  mesh_gpu: Box<MeshGPUInstance>,
  instance_gpu: GPUBufferResourceView,
}

impl Stream for TransformInstanceGPU {
  type Item = RenderComponentDeltaFlag;
  fn poll_next(self: Pin<&mut Self>, _: &mut Context) -> Poll<Option<Self::Item>> {
    Poll::Pending
  }
}

only_vertex!(TransformInstanceMat, Mat4<f32>);

#[repr(C)]
#[derive(Clone, Copy, shadergraph::ShaderVertex)]
pub struct ShaderMat4VertexInput {
  #[semantic(TransformInstanceMat)]
  mat: Mat4<f32>,
}

impl ShaderGraphProvider for TransformInstanceGPU {
  fn build(
    &self,
    builder: &mut ShaderGraphRenderPipelineBuilder,
  ) -> Result<(), ShaderGraphBuildError> {
    self.mesh_gpu.build(builder)?;
    builder.vertex(|builder, _| {
      builder.register_vertex::<ShaderMat4VertexInput>(VertexStepMode::Instance);

      let world_mat = builder.query::<TransformInstanceMat>()?;
      let world_normal_mat: Node<Mat3<f32>> = world_mat.into();

      if let Ok(position) = builder.query::<GeometryPosition>() {
        builder.register::<GeometryPosition>((world_mat * (position, 1.).into()).xyz());
      }

      if let Ok(normal) = builder.query::<GeometryNormal>() {
        builder.register::<GeometryNormal>(world_normal_mat * normal);
      }

      Ok(())
    })
  }
}

impl ShaderHashProvider for TransformInstanceGPU {
  fn hash_pipeline(&self, hasher: &mut PipelineHasher) {
    self.mesh_gpu.hash_pipeline(hasher)
  }
}

impl ShaderPassBuilder for TransformInstanceGPU {
  fn setup_pass(&self, ctx: &mut GPURenderPassCtx) {
    self.mesh_gpu.setup_pass(ctx);
    ctx.set_vertex_buffer_owned_next(&self.instance_gpu);
  }
}

impl ReactiveRenderComponentSource for ReactiveMeshGPUOf<TransformInstancedSceneMesh> {
  fn as_reactive_component(&self) -> &dyn ReactiveRenderComponent {
    self.as_ref() as &dyn ReactiveRenderComponent
  }
}

impl WebGPUMesh for TransformInstancedSceneMesh {
  type ReactiveGPU =
    impl AsRef<RenderComponentCell<TransformInstanceGPU>> + Stream<Item = RenderComponentDeltaFlag>;

  fn create_reactive_gpu(
    source: &SceneItemRef<Self>,
    ctx: &ShareBindableResourceCtx,
  ) -> Self::ReactiveGPU {
    let weak = source.downgrade();
    let ctx = ctx.clone();

    let create = move || {
      if let Some(m) = weak.upgrade() {
        let mesh = m.read();
        // todo, current we do not support reuse this inner mesh!
        let mesh_gpu = mesh.mesh.create_scene_reactive_gpu(&ctx).unwrap();
        let mesh_gpu = Box::new(mesh_gpu);

        let instance_gpu = create_gpu_buffer(
          bytemuck::cast_slice(mesh.transforms.as_slice()),
          webgpu::BufferUsages::VERTEX,
          &ctx.gpu.device,
        )
        .create_default_view();

        let r = TransformInstanceGPU {
          mesh_gpu,
          instance_gpu,
        };
        Some(r)
      } else {
        None
      }
    };

    let gpu = create().unwrap();
    let state = RenderComponentCell::new(gpu);

    source
      .single_listen_by::<()>(any_change_no_init)
      .fold_signal(state, move |_, state| {
        if let Some(gpu) = create() {
          state.inner = gpu;
          RenderComponentDeltaFlag::all().into()
        } else {
          None
        }
      })
  }

  fn draw_impl(&self, group: MeshDrawGroup) -> DrawCommand {
    let mut inner = self.mesh.draw_impl(group);
    match &mut inner {
      DrawCommand::Indexed { instances, .. } => {
        assert_eq!(*instances, 0..1);
        *instances = 0..self.transforms.len() as u32;
      }
      DrawCommand::Array { instances, .. } => {
        assert_eq!(*instances, 0..1);
        *instances = 0..self.transforms.len() as u32;
      }
      DrawCommand::Skip => {}
    }
    inner
  }

  fn topology(&self) -> webgpu::PrimitiveTopology {
    self.mesh.topology()
  }

  fn try_pick(&self, f: &mut dyn FnMut(&dyn IntersectAbleGroupedMesh)) {
    self.mesh.try_pick(&mut |target| {
      let wrapped = InstanceTransformedPickImpl {
        mat: &self.transforms,
        mesh: target,
      };
      f(&wrapped as &dyn IntersectAbleGroupedMesh)
    });
  }
}

struct InstanceTransformedPickImpl<'a> {
  pub mat: &'a [Mat4<f32>],
  pub mesh: &'a dyn IntersectAbleGroupedMesh,
}

impl<'a> IntersectAbleGroupedMesh for InstanceTransformedPickImpl<'a> {
  fn intersect_list(
    &self,
    ray: Ray3,
    conf: &MeshBufferIntersectConfig,
    result: &mut MeshBufferHitList,
    group: MeshDrawGroup,
  ) {
    self.mat.iter().for_each(|mat| {
      let world_inv = mat.inverse_or_identity();
      let local_ray = ray.clone().apply_matrix_into(world_inv);
      self.mesh.intersect_list(local_ray, conf, result, group)
    })
  }

  fn intersect_nearest(
    &self,
    ray: Ray3,
    conf: &MeshBufferIntersectConfig,
    group: MeshDrawGroup,
  ) -> OptionalNearest<MeshBufferHitPoint> {
    self
      .mat
      .iter()
      .fold(OptionalNearest::none(), |mut pre, mat| {
        let world_inv = mat.inverse_or_identity();
        let local_ray = ray.clone().apply_matrix_into(world_inv);
        let r = self.mesh.intersect_nearest(local_ray, conf, group);
        *pre.refresh_nearest(r)
      })
  }
}
