use crate::*;

pub trait SurfaceProvider {
  fn create_surface(&self, instance: &gpu::Instance) -> Result<gpu::Surface, CreateSurfaceError>;
  fn size(&self) -> Size;
}

impl SurfaceProvider for winit::window::Window {
  fn create_surface(&self, instance: &gpu::Instance) -> Result<gpu::Surface, CreateSurfaceError> {
    unsafe { instance.create_surface(self) }
  }

  fn size(&self) -> Size {
    let size = self.inner_size();
    Size::from_u32_pair_min_one((size.width, size.height))
  }
}

pub struct GPUSurface {
  pub surface: gpu::Surface,
  pub config: gpu::SurfaceConfiguration,
  pub capabilities: gpu::SurfaceCapabilities,
  pub size: Size,
}

impl GPUSurface {
  #[allow(clippy::or_fun_call)]
  pub(crate) fn new(
    adapter: &gpu::Adapter,
    device: &GPUDevice,
    surface: gpu::Surface,
    init_resolution: Size,
  ) -> Self {
    let capabilities = surface.get_capabilities(adapter);
    let swapchain_format = capabilities
      .formats
      .iter()
      .find(|&f| *f == gpu::TextureFormat::Bgra8UnormSrgb) // prefer use srgb
      .or(capabilities.formats.first())
      .expect("should at least one supported format");

    let config = gpu::SurfaceConfiguration {
      usage: gpu::TextureUsages::RENDER_ATTACHMENT,
      format: *swapchain_format,
      view_formats: vec![*swapchain_format],
      width: Into::<usize>::into(init_resolution.width) as u32,
      height: Into::<usize>::into(init_resolution.height) as u32,
      present_mode: gpu::PresentMode::AutoVsync,
      alpha_mode: gpu::CompositeAlphaMode::Auto,
    };

    surface.configure(device, &config);

    Self {
      capabilities,
      surface,
      config,
      size: init_resolution,
    }
  }

  pub fn resize(&mut self, size: Size, device: &GPUDevice) {
    self.config.width = Into::<usize>::into(size.width) as u32;
    self.config.height = Into::<usize>::into(size.height) as u32;
    self.surface.configure(device, &self.config);
    self.size = size;
  }

  pub fn get_current_frame(&self) -> Result<gpu::SurfaceTexture, gpu::SurfaceError> {
    self.surface.get_current_texture()
  }

  pub fn get_current_frame_with_render_target_view(
    &self,
  ) -> Result<(gpu::SurfaceTexture, RenderTargetView), gpu::SurfaceError> {
    let frame = self.get_current_frame()?;

    let view = frame
      .texture
      .create_view(&gpu::TextureViewDescriptor::default());
    let view = Arc::new(view);

    Ok((
      frame,
      RenderTargetView::SurfaceTexture {
        view,
        size: self.size,
        format: self.config.format,
        view_id: get_resource_view_guid(),
        bindgroup_holder: Default::default(),
      },
    ))
  }
}
