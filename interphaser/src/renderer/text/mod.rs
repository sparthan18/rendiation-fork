mod gpu_renderer;

use std::{
  collections::hash_map::DefaultHasher,
  hash::{Hash, Hasher},
};

use glyph_brush::{HorizontalAlign, VerticalAlign};
use gpu_renderer::*;
use rendiation_algebra::Vec2;
use rendiation_texture::Size;
use rendiation_texture_packer::etagere_wrap::EtagerePacker;
use rendiation_webgpu::{GPURenderPass, GPU};

pub mod cache_glyph;
pub use cache_glyph::*;

pub mod cache_text;
pub use cache_text::*;

pub mod gpu_cache;
pub use gpu_cache::*;

pub mod layout;
pub use layout::*;

pub mod raster;
pub use raster::*;

pub mod packer;
pub use packer::*;

use crate::{Color, FontManager, LayoutSize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum LineWrap {
  Single,
  Multiple,
}

impl Default for LineWrap {
  fn default() -> Self {
    Self::Single
  }
}

#[derive(Debug, Clone)]
pub struct TextInfo {
  pub content: String,
  pub bounds: LayoutSize,
  pub line_wrap: LineWrap,
  pub horizon_align: HorizontalAlign,
  pub vertical_align: VerticalAlign,
  pub color: Color,
  pub font_size: f32,
  pub x: f32,
  pub y: f32,
}

pub type TextHash = u64;

impl TextInfo {
  pub fn hash(&self) -> TextHash {
    let mut hasher = DefaultHasher::default();
    self.content.hash(&mut hasher);
    self.bounds.width.to_bits().hash(&mut hasher);
    self.bounds.height.to_bits().hash(&mut hasher);
    self.line_wrap.hash(&mut hasher);
    self.horizon_align.hash(&mut hasher);
    self.vertical_align.hash(&mut hasher);
    self.color.r.to_bits().hash(&mut hasher);
    self.color.g.to_bits().hash(&mut hasher);
    self.color.b.to_bits().hash(&mut hasher);
    self.font_size.to_bits().hash(&mut hasher);
    self.x.to_bits().hash(&mut hasher);
    self.y.to_bits().hash(&mut hasher);
    hasher.finish()
  }
}

pub struct TextRenderer {
  renderer: TextWebGPURenderer,
  gpu_texture_cache: WebGPUTextureCache,
  gpu_vertex_cache: WebGPUTextCache,

  cache: TextCache,
}

impl TextRenderer {
  pub fn new(
    device: &wgpu::Device,
    filter_mode: wgpu::FilterMode,
    render_format: wgpu::TextureFormat,
  ) -> Self {
    let init_size = Size::from_usize_pair_min_one((512, 512));
    let tolerance = Default::default();

    let texture_cache = WebGPUTextureCache::init(init_size, device);

    let raster = AbGlyphRaster::default();

    let packer = EtagerePacker::default();

    let glyph_cache = GlyphCache::new(init_size, tolerance, raster, packer);

    let text_cache = TextCache::new(glyph_cache, GlyphBrushLayouter::default());

    Self {
      renderer: TextWebGPURenderer::new(
        device,
        filter_mode,
        render_format,
        Vec2::new(1000., 1000.),
        texture_cache.get_view(),
      ),
      gpu_texture_cache: texture_cache,
      gpu_vertex_cache: Default::default(),
      cache: text_cache,
    }
  }

  pub fn resize_view(&mut self, size: Vec2<f32>, queue: &wgpu::Queue) {
    self.renderer.resize_view(size, queue)
  }

  pub fn draw_gpu_text<'a>(&'a self, pass: &mut GPURenderPass<'a>, text: TextHash) {
    if let Some(gpu_text) = self.gpu_vertex_cache.get_cache(text) {
      self.renderer.draw(pass, gpu_text)
    }
  }

  pub fn queue_text(&mut self, text: &TextInfo, fonts: &FontManager) -> Option<TextHash> {
    (!text.content.is_empty()).then(|| self.cache.queue(text, fonts))
  }

  pub fn drop_cache(&mut self, text: TextHash) {
    self.cache.drop_cache(text);
    self.gpu_vertex_cache.drop_cache(text);
  }

  pub fn clear_cache(&mut self) {
    self.cache.clear_cache();
    self.gpu_vertex_cache.clear_cache();
  }

  pub fn process_queued(&mut self, gpu: &GPU, fonts: &FontManager) {
    self.cache.process_queued(
      fonts,
      |action| match action {
        TextureCacheAction::ResizeTo(new_size) => {
          if usize::from(new_size.width) > 4096 || usize::from(new_size.height) > 4096 {
            return false;
          }
          let device = &gpu.device;
          self.gpu_texture_cache = WebGPUTextureCache::init(new_size, device);
          self
            .renderer
            .cache_resized(device, self.gpu_texture_cache.get_view());
          true
        }
        TextureCacheAction::UpdateAt { data, range } => {
          self
            .gpu_texture_cache
            .update_texture(data, range, &gpu.queue);
          true
        }
      },
      |hash, data| {
        self
          .gpu_vertex_cache
          .add_cache(hash, create_gpu_text(&gpu.device, data.as_slice()).unwrap())
      },
    );
  }
}
