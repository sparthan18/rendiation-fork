use crate::*;

// struct SceneNodeGPUSystem;
// struct SceneCameraGPUSystem;
// struct SceneBundleGPUSystem;

#[pin_project::pin_project]
pub struct SceneGPUSystem {
  // nodes: SceneNodeGPUSystem,
  // // the camera gpu data are mostly related to scene node it used, so keep it at scene level;
  // cameras: SceneCameraGPUSystem,
  // bundle: SceneBundleGPUSystem,
  #[pin]
  models: StreamMap<ReactiveSceneModelGPUType>,

  #[pin]
  source: SceneGPUUpdateSource,

  pub cameras: RefCell<CameraGPUMap>,
  pub nodes: RefCell<NodeGPUMap>,
  pub lights: RefCell<GPULightCache>,
}

impl SceneGPUSystem {
  pub fn encode(&self, _encoder: &mut GPUCommandEncoder, _pass_dispatcher: &dyn RenderComponent) {
    // do encoding
  }
}

impl Stream for SceneGPUSystem {
  type Item = ();

  fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
    let this = self.project();
    early_return_ready!(this.source.poll_next(cx));
    this.models.poll_next(cx).map(|v| v.map(|_| {}))
  }
}
type SceneGPUUpdateSource = impl Stream<Item = ()> + Unpin;

impl SceneGPUSystem {
  pub fn new(scene: &Scene, contents: Arc<RwLock<ContentGPUSystem>>) -> Self {
    let models = Default::default();
    let contents_c = contents.clone();

    let source = scene.unbound_listen_by(all_delta).map(move |delta| {
      let contents = contents_c.write().unwrap();
      let mut models = contents.models.write().unwrap();
      if let SceneInnerDelta::models(delta) = delta {
        match delta {
          arena::ArenaDelta::Mutate((model, _)) => {
            models.remove(model.id());
            models.get_or_insert_with(model.id(), || {
              //
              todo!()
            });
          }
          arena::ArenaDelta::Insert((model, _)) => {
            models.get_or_insert_with(model.id(), || {
              //
              todo!()
            });
          }
          arena::ArenaDelta::Remove(handle) => {
            models.remove(handle.index());
          }
        }
      }
    });

    Self {
      models,
      // nodes: (),
      // cameras: (),
      // bundle: (),
      source,
      cameras: Default::default(),
      nodes: Default::default(),
      lights: Default::default(),
    }
  }

  pub fn maintain(&mut self) {
    do_updates(self, |_| {});
  }
}
