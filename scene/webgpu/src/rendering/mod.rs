pub mod forward;
pub use forward::*;

pub mod list;
pub use list::*;

pub mod copy_frame;
pub use copy_frame::*;
pub mod highlight;
pub use highlight::*;
pub mod background;
pub use background::*;
pub mod quad;
pub use quad::*;
use webgpu::{GPURenderPass, GPURenderPassCtx};

pub mod framework;
pub use framework::*;

use crate::{DefaultPassDispatcher, GPUResourceCache, Scene, SceneCamera, WebGPUScene};

pub struct SceneRenderPass<'a, 'b, 'c> {
  pub ctx: GPURenderPassCtx<'a, 'b>,
  pub resources: &'c mut GPUResourceCache,
}

impl<'a, 'b, 'c> SceneRenderPass<'a, 'b, 'c> {
  pub fn default_dispatcher(&self) -> DefaultPassDispatcher {
    DefaultPassDispatcher {
      formats: self.ctx.pass.formats().clone(),
    }
  }
}

impl<'a, 'b, 'c> std::ops::Deref for SceneRenderPass<'a, 'b, 'c> {
  type Target = GPURenderPass<'a>;

  fn deref(&self) -> &Self::Target {
    &self.ctx.pass
  }
}

impl<'a, 'b, 'c> std::ops::DerefMut for SceneRenderPass<'a, 'b, 'c> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.ctx.pass
  }
}

pub struct CameraRef<'a, T> {
  camera: &'a SceneCamera,
  inner: T,
}

pub trait WebGPUScenePipelineHelper {
  fn by_main_camera<T>(&self, inner: T) -> CameraRef<T>;
  fn by_main_camera_and_self<T>(&self, inner: T) -> CameraSceneRef<T>;
}

impl WebGPUScenePipelineHelper for Scene<WebGPUScene> {
  fn by_main_camera<T>(&self, inner: T) -> CameraRef<T> {
    CameraRef {
      camera: self.active_camera.as_ref().unwrap(),
      inner,
    }
  }

  fn by_main_camera_and_self<T>(&self, inner: T) -> CameraSceneRef<T> {
    CameraSceneRef {
      camera: self.active_camera.as_ref().unwrap(),
      scene: self,
      inner,
    }
  }
}

impl<'a, T: PassContentWithCamera> PassContent for CameraRef<'a, T> {
  fn render(&mut self, pass: &mut SceneRenderPass) {
    self.inner.render(pass, self.camera);
  }
}

pub trait PassContentWithCamera {
  fn render(&mut self, pass: &mut SceneRenderPass, camera: &SceneCamera);
}

pub trait PassContentWithSceneAndCamera {
  fn render(
    &mut self,
    pass: &mut SceneRenderPass,
    scene: &Scene<WebGPUScene>,
    camera: &SceneCamera,
  );
}

pub struct CameraSceneRef<'a, T> {
  camera: &'a SceneCamera,
  scene: &'a Scene<WebGPUScene>,
  inner: T,
}

impl<'a, T: PassContentWithSceneAndCamera> PassContent for CameraSceneRef<'a, T> {
  fn render(&mut self, pass: &mut SceneRenderPass) {
    self.inner.render(pass, self.scene, self.camera);
  }
}
