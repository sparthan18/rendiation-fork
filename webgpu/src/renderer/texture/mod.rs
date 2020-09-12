use crate::renderer::buffer::WGPUBuffer;
use crate::renderer::WGPURenderer;

pub mod texture_cube;
pub mod texture_dimension;
pub mod texture_format;
use crate::renderer::texture_dimension::*;
use crate::renderer::texture_format::*;

pub struct WGPUTexture<V: TextureDimension = TextureSize2D> {
  gpu_texture: wgpu::Texture,
  descriptor: wgpu::TextureDescriptor<'static>,
  size: V,
  view: wgpu::TextureView,
  format: TextureFormat,
}

impl WGPUTexture {
  pub fn new_as_depth(
    renderer: &WGPURenderer,
    format: wgpu::TextureFormat,
    size: (usize, usize),
  ) -> Self {
    let size: TextureSize2D = size.into();
    let descriptor = wgpu::TextureDescriptor {
      label: None,
      size: size.to_wgpu(),
      array_layer_count: 1,
      mip_level_count: 1,
      sample_count: 1,
      dimension: TextureSize2D::WGPU_CONST,
      format,
      usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
    };
    let depth_texture = renderer.device.create_texture(&descriptor);
    let view = depth_texture.create_default_view();
    Self {
      descriptor,
      gpu_texture: depth_texture,
      view,
      size,
      format: TextureFormat::Rgba8UnormSrgb,
    }
  }

  pub fn new_as_target_default(renderer: &WGPURenderer, size: (usize, usize)) -> Self {
    WGPUTexture::new_as_target(renderer, TextureFormat::Rgba8UnormSrgb, size)
  }

  pub fn new_as_target(
    renderer: &WGPURenderer,
    format: TextureFormat,
    size: (usize, usize),
  ) -> Self {
    let size: TextureSize2D = size.into();
    let descriptor = wgpu::TextureDescriptor {
      label: None,
      size: size.to_wgpu(),
      array_layer_count: 1,
      mip_level_count: 1,
      sample_count: 1,
      dimension: TextureSize2D::WGPU_CONST,
      format: format.get_wgpu_format(),
      usage: wgpu::TextureUsage::SAMPLED
        | wgpu::TextureUsage::COPY_DST
        | wgpu::TextureUsage::OUTPUT_ATTACHMENT,
    };
    let gpu_texture = renderer.device.create_texture(&descriptor);
    let view = gpu_texture.create_default_view();
    Self {
      gpu_texture,
      descriptor,
      view,
      size,
      format,
    }
  }

  pub fn new_from_image_data(
    renderer: &mut WGPURenderer,
    data: &[u8],
    size: (u32, u32, u32),
  ) -> WGPUTexture {
    let (width, height, depth) = size;
    let descriptor = wgpu::TextureDescriptor {
      label: None,
      size: wgpu::Extent3d {
        width,
        height,
        depth,
      },
      array_layer_count: 1,
      mip_level_count: 1,
      sample_count: 1,
      dimension: TextureSize2D::WGPU_CONST,
      format: wgpu::TextureFormat::Rgba8UnormSrgb,
      usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
    };
    let gpu_texture = renderer.device.create_texture(&descriptor);
    let view = gpu_texture.create_default_view();
    let wgpu_texture = Self {
      gpu_texture,
      descriptor,
      view,
      size: TextureSize2D {
        width: size.0 as u32,
        height: size.1 as u32,
      },
      format: TextureFormat::Rgba8UnormSrgb,
    };

    wgpu_texture.upload(renderer, data);
    wgpu_texture
  }

  pub fn view(&self) -> &wgpu::TextureView {
    &self.view
  }

  pub fn size(&self) -> TextureSize2D {
    self.size
  }

  pub fn format(&self) -> &wgpu::TextureFormat {
    &self.descriptor.format
  }

  /// this will not keep content resize, just recreate the gpu resource with new size
  pub fn resize(&mut self, renderer: &WGPURenderer, size: (usize, usize)) {
    self.descriptor.size.width = size.0 as u32;
    self.descriptor.size.height = size.1 as u32;
    self.gpu_texture = renderer.device.create_texture(&self.descriptor);
    self.view = self.gpu_texture.create_default_view();
  }

  fn upload(&self, renderer: &mut WGPURenderer, image_data: &[u8]) {
    upload(renderer, &self, image_data, 0)
  }
}

impl<V: TextureDimension> WGPUTexture<V> {
  pub async fn read(
    &self,
    renderer: &mut WGPURenderer,
  ) -> Result<wgpu::BufferReadMapping, wgpu::BufferAsyncErr> {
    let pixel_count = self.size.get_pixel_size() as u64;
    let data_size = pixel_count * self.format.get_pixel_data_stride() as u64;

    let output_buffer = renderer.device.create_buffer(&wgpu::BufferDescriptor {
      label: None,
      size: data_size,
      usage: wgpu::BufferUsage::MAP_READ | wgpu::BufferUsage::COPY_DST,
    });

    let buffer_future = output_buffer.map_read(0, data_size);

    renderer.device.poll(wgpu::Maintain::Wait);

    buffer_future.await
  }
}

pub fn upload(
  renderer: &mut WGPURenderer,
  texture: &WGPUTexture,
  image_data: &[u8],
  target_layer: u32,
) {
  let buffer = WGPUBuffer::new(renderer, image_data, wgpu::BufferUsage::COPY_SRC);

  renderer.encoder.copy_buffer_to_texture(
    wgpu::BufferCopyView {
      buffer: buffer.get_gpu_buffer(),
      offset: 0,
      bytes_per_row: 4 * texture.descriptor.size.width,
      rows_per_image: 0,
    },
    wgpu::TextureCopyView {
      texture: &texture.gpu_texture,
      mip_level: 0,
      array_layer: target_layer,
      origin: wgpu::Origin3d::ZERO,
    },
    texture.descriptor.size,
  );
}