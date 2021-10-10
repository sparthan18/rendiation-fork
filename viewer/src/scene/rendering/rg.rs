use std::{cell::RefCell, collections::HashMap, rc::Rc};

use rendiation_algebra::Vec3;
use rendiation_texture::Size;
use rendiation_webgpu::*;

use crate::{
  RenderPassDispatcher, Scene, StandardForward, ViewerRenderPass, ViewerRenderPassCreator,
};

pub struct ResourcePoolInner {
  pub attachments: HashMap<(Size, wgpu::TextureFormat), Vec<wgpu::Texture>>,
}

#[derive(Clone)]
pub struct ResourcePool {
  pub inner: Rc<RefCell<ResourcePoolInner>>,
}

impl Default for ResourcePool {
  fn default() -> Self {
    todo!()
  }
}

pub struct RenderEngine {
  resource: ResourcePool,
  gpu: GPU,
  output_size: Size,
  output: wgpu::TextureView,
}

impl RenderEngine {
  pub fn screen(&self) -> Attachment<wgpu::TextureFormat> {
    todo!()
  }
}

pub fn attachment() -> AttachmentDescriptor<wgpu::TextureFormat> {
  AttachmentDescriptor {
    format: wgpu::TextureFormat::Rgba8Unorm,
    sizer: default_sizer(),
  }
}

pub fn depth_attachment() -> AttachmentDescriptor<wgpu::TextureFormat> {
  AttachmentDescriptor {
    format: wgpu::TextureFormat::Depth24PlusStencil8,
    sizer: default_sizer(),
  }
}

pub trait AttachmentFormat: Into<wgpu::TextureFormat> + Copy {}
impl<T: Into<wgpu::TextureFormat> + Copy> AttachmentFormat for T {}

#[derive(Clone)]
pub struct Attachment<F: AttachmentFormat> {
  pool: ResourcePool,
  des: AttachmentDescriptor<F>,
  size: Size,
  texture: Option<Rc<wgpu::Texture>>,
}

impl<F: AttachmentFormat> Drop for Attachment<F> {
  fn drop(&mut self) {
    if let Ok(texture) = Rc::try_unwrap(self.texture.take().unwrap()) {
      let mut pool = self.pool.inner.borrow_mut();
      let cached = pool
        .attachments
        .entry((self.size, self.des.format.into()))
        .or_insert_with(Default::default);

      cached.push(texture)
    }
  }
}

pub struct AttachmentDescriptor<F> {
  format: F,
  sizer: Box<dyn Fn(Size) -> Size>,
}

impl<F> Clone for AttachmentDescriptor<F> {
  fn clone(&self) -> Self {
    todo!()
  }
}

fn default_sizer() -> Box<dyn Fn(Size) -> Size> {
  Box::new(|size| size)
}

impl<F: AttachmentFormat> AttachmentDescriptor<F> {
  pub fn format(mut self, format: F) -> Self {
    self.format = format;
    self
  }
}

impl<F: AttachmentFormat> AttachmentDescriptor<F> {
  // #[track_caller]
  pub fn request(self, engine: &RenderEngine) -> Attachment<F> {
    let size = (self.sizer)(engine.output_size);
    let mut resource = engine.resource.inner.borrow_mut();
    let cached = resource
      .attachments
      .entry((size, self.format.into()))
      .or_insert_with(Default::default);
    let texture = cached.pop().unwrap_or_else(|| {
      engine.gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: size.into_gpu_size(),
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: self.format.into(),
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
      })
    });
    Attachment {
      pool: engine.resource.clone(),
      des: self,
      size,
      texture: Rc::new(texture).into(),
    }
  }
}

pub trait Pipeline {
  fn render(&mut self, engine: &RenderEngine, scene: &SceneDispatcher);
}

pub struct HighLight {
  color: Vec3<f32>,
}

impl ViewerRenderPass for HighLight {
  fn depth_stencil_format(&self) -> Option<wgpu::TextureFormat> {
    wgpu::TextureFormat::Depth32Float.into()
  }

  fn color_format(&self) -> &[wgpu::TextureFormat] {
    // self.color_format.as_slice()
    todo!()
  }
}

impl ViewerRenderPassCreator for HighLight {
  type TargetResource = wgpu::TextureView;

  fn create_pass<'a>(
    &'a self,
    scene: &Scene,
    target: &'a Self::TargetResource,
    encoder: &'a mut wgpu::CommandEncoder,
  ) -> wgpu::RenderPass<'a> {
    todo!()
  }
}

pub struct SimplePipeline {
  forward: StandardForward,
  highlight: HighLight,
}

impl Scene {
  pub fn create_pass<P>(&mut self, pass: &mut P) -> RenderPassDispatcher<P> {
    // RenderPassDispatcher {
    //     scene: self,
    //     pass,
    //   }
    todo!()
  }
}

pub struct SceneDispatcher {
  scene: Rc<RefCell<Scene>>,
}

impl SceneDispatcher {
  pub fn create_content<T>(&self, test: &mut T) -> impl PassContent {
    ForwardPass
  }
}

pub struct ForwardPass;

impl PassContent for ForwardPass {
  fn update(&mut self, gpu: &GPU, scene: &mut Scene, resource: &mut ResourcePoolInner) {
    todo!()
  }

  fn setup_pass<'a>(
    &'a self,
    pass: &mut wgpu::RenderPass<'a>,
    scene: &'a Scene,
    resource: &'a ResourcePoolInner,
  ) {
    todo!()
  }
}

pub trait PassContent: 'static {
  fn update(&mut self, gpu: &GPU, scene: &mut Scene, resource: &mut ResourcePoolInner);
  fn setup_pass<'a>(
    &'a self,
    pass: &mut wgpu::RenderPass<'a>,
    scene: &'a Scene,
    resource: &'a ResourcePoolInner,
  );
}

impl Pipeline for SimplePipeline {
  #[rustfmt::skip]
  fn render(&mut self, engine: &RenderEngine, scene: &SceneDispatcher) {
    let scene_main_content = scene.create_content(&mut self.forward);

    let mut scene_color = attachment()
      .format(wgpu::TextureFormat::Rgba8Unorm)
      .request(engine);

    let mut scene_depth = depth_attachment()
      .format(wgpu::TextureFormat::Depth32Float)
      .request(engine);

    pass("scene_pass")
      .with_color(&mut scene_color, clear(color(0.1, 0.2, 0.3)))
      .with_depth(&mut scene_depth, clear(1.))
      .render_by(scene_main_content)
      .run(engine, scene);

    let mut high_light_object_mask = attachment()
      .format(wgpu::TextureFormat::Rgba8Unorm)
      .request(engine);

    let high_light_object = scene.create_content(&mut self.highlight);

    pass("high_light_pass")
      .with_color(&mut high_light_object_mask, clear(color_same(1.)))
      .render_by(high_light_object)
      .run(engine, scene);

    pass("final_compose")
      // .with_color(&mut scene_color, clear(color(0.1, 0.2, 0.3)))
      .with_color(&mut engine.screen(), clear(color_same(1.)))
      .render_by(copy(scene_color))
      .render_by(high_light_blend(high_light_object_mask))
      .run(engine, scene);
  }
}

pub struct HiLighter<'a> {
  source: &'a mut Attachment<wgpu::TextureFormat>,
}

impl<'a> Renderable for HiLighter<'a> {
  fn setup_pass<'r>(&'r self, pass: &mut wgpu::RenderPass<'r>) {
    todo!()
  }
}

pub fn high_light_blend(source: Attachment<wgpu::TextureFormat>) -> impl PassContent {
  ForwardPass
}

pub struct Copier<'a> {
  source: &'a mut Attachment<wgpu::TextureFormat>,
}

impl<'a> Renderable for Copier<'a> {
  fn setup_pass<'r>(&'r self, pass: &mut wgpu::RenderPass<'r>) {
    todo!()
  }
}

pub fn copy(source: Attachment<wgpu::TextureFormat>) -> impl PassContent {
  ForwardPass
}

pub fn pass(name: &'static str) -> PassDescriptor {
  PassDescriptor {
    name,
    channels: Vec::new(),
    tasks: Vec::new(),
    depth_stencil_target: None,
  }
}

pub struct PassDescriptor {
  name: &'static str,
  channels: Vec<(
    wgpu::Operations<wgpu::Color>,
    Attachment<wgpu::TextureFormat>,
  )>,
  tasks: Vec<Box<dyn PassContent>>,
  depth_stencil_target: Option<(wgpu::Operations<f32>, Attachment<wgpu::TextureFormat>)>,
}

impl ViewerRenderPass for PassDescriptor {
  fn depth_stencil_format(&self) -> Option<wgpu::TextureFormat> {
    todo!()
  }

  fn color_format(&self) -> &[wgpu::TextureFormat] {
    // self.color_format.as_slice()
    todo!()
  }
}

impl ViewerRenderPassCreator for PassDescriptor {
  type TargetResource = wgpu::TextureView;

  fn create_pass<'a>(
    &'a self,
    scene: &Scene,
    target: &'a Self::TargetResource,
    encoder: &'a mut wgpu::CommandEncoder,
  ) -> wgpu::RenderPass<'a> {
    todo!()
  }
}

impl PassDescriptor {
  #[must_use]
  pub fn with_color(
    mut self,
    attachment: &mut Attachment<wgpu::TextureFormat>,
    op: impl Into<wgpu::Operations<wgpu::Color>>,
  ) -> Self {
    self.channels.push((op.into(), attachment.clone()));
    self
  }

  #[must_use]
  pub fn with_depth(
    mut self,
    attachment: &mut Attachment<wgpu::TextureFormat>,
    op: impl Into<wgpu::Operations<f32>>,
  ) -> Self {
    self
      .depth_stencil_target
      .replace((op.into(), attachment.clone()));
    self
  }

  #[must_use]
  pub fn render_by(mut self, renderable: impl PassContent) -> Self {
    self.tasks.push(Box::new(renderable));
    self
  }

  pub fn run(self, engine: &RenderEngine, scene: &SceneDispatcher) {
    let mut resource = engine.resource.inner.borrow_mut();

    let mut encoder = engine.gpu.encoder.borrow_mut();

    let color_attachments: Vec<_> = self
      .channels
      .iter()
      .map(|(ops, attachment)| wgpu::RenderPassColorAttachment {
        view: &attachment.texture.as_ref().unwrap().create_view(todo!()),
        resolve_target: None,
        ops: *ops,
      })
      .collect();

    let depth_stencil_attachment =
      self
        .depth_stencil_target
        .map(|(ops, attachment)| wgpu::RenderPassDepthStencilAttachment {
          view: &attachment.texture.as_ref().unwrap().create_view(todo!()),
          depth_ops: ops.into(),
          stencil_ops: None,
        });

    let scene = scene.scene.borrow();
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: self.name.into(),
      color_attachments: color_attachments.as_slice(),
      depth_stencil_attachment,
    });

    for task in &self.tasks {
      task.setup_pass(&mut pass, &scene, &resource)
    }
  }
}

pub fn color(r: f32, g: f32, b: f32) -> wgpu::Color {
  todo!()
}

pub fn color_same(r: f32) -> wgpu::Color {
  // or use marco?
  todo!()
}

pub fn clear<V>(v: V) -> Operations<V> {
  wgpu::Operations {
    load: wgpu::LoadOp::Clear(v),
    store: true,
  }
}
