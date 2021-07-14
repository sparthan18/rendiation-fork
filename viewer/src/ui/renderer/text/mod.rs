mod cache;
use std::borrow::Cow;

mod pipeline;
use glyph_brush::{
  ab_glyph::{self, Rect},
  BrushAction, BrushError, DefaultSectionHasher, Extra, FontId, GlyphBrushBuilder, GlyphCruncher,
  GlyphPositioner, Section, SectionGlyph,
};
use pipeline::*;

pub struct TextRenderer {
  pipeline: Pipeline<()>,
  glyph_brush: glyph_brush::GlyphBrush<Instance, Extra, ab_glyph::FontArc, DefaultSectionHasher>,
}

impl TextRenderer {
  /// Queues a section/layout to be drawn by the next call of
  /// [`draw_queued`](struct.GlyphBrush.html#method.draw_queued). Can be
  /// called multiple times to queue multiple sections for drawing.
  ///
  /// Benefits from caching, see [caching behaviour](#caching-behaviour).
  #[inline]
  pub fn queue<'a, S>(&mut self, section: S)
  where
    S: Into<Cow<'a, Section<'a>>>,
  {
    self.glyph_brush.queue(section)
  }

  /// Queues a section/layout to be drawn by the next call of
  /// [`draw_queued`](struct.GlyphBrush.html#method.draw_queued). Can be
  /// called multiple times to queue multiple sections for drawing.
  ///
  /// Used to provide custom `GlyphPositioner` logic, if using built-in
  /// [`Layout`](enum.Layout.html) simply use
  /// [`queue`](struct.GlyphBrush.html#method.queue)
  ///
  /// Benefits from caching, see [caching behaviour](#caching-behaviour).
  #[inline]
  pub fn queue_custom_layout<'a, S, G>(&mut self, section: S, custom_layout: &G)
  where
    G: GlyphPositioner,
    S: Into<Cow<'a, Section<'a>>>,
  {
    self.glyph_brush.queue_custom_layout(section, custom_layout)
  }

  /// Queues pre-positioned glyphs to be processed by the next call of
  /// [`draw_queued`](struct.GlyphBrush.html#method.draw_queued). Can be
  /// called multiple times.
  #[inline]
  pub fn queue_pre_positioned(
    &mut self,
    glyphs: Vec<SectionGlyph>,
    extra: Vec<Extra>,
    bounds: Rect,
  ) {
    self.glyph_brush.queue_pre_positioned(glyphs, extra, bounds)
  }

  /// Retains the section in the cache as if it had been used in the last
  /// draw-frame.
  ///
  /// Should not be necessary unless using multiple draws per frame with
  /// distinct transforms, see [caching behaviour](#caching-behaviour).
  #[inline]
  pub fn keep_cached_custom_layout<'a, S, G>(&mut self, section: S, custom_layout: &G)
  where
    S: Into<Cow<'a, Section<'a>>>,
    G: GlyphPositioner,
  {
    self
      .glyph_brush
      .keep_cached_custom_layout(section, custom_layout)
  }

  /// Retains the section in the cache as if it had been used in the last
  /// draw-frame.
  ///
  /// Should not be necessary unless using multiple draws per frame with
  /// distinct transforms, see [caching behaviour](#caching-behaviour).
  #[inline]
  pub fn keep_cached<'a, S>(&mut self, section: S)
  where
    S: Into<Cow<'a, Section<'a>>>,
  {
    self.glyph_brush.keep_cached(section)
  }

  /// Returns the available fonts.
  ///
  /// The `FontId` corresponds to the index of the font data.
  #[inline]
  pub fn fonts(&self) -> &[ab_glyph::FontArc] {
    self.glyph_brush.fonts()
  }

  /// Adds an additional font to the one(s) initially added on build.
  ///
  /// Returns a new [`FontId`](struct.FontId.html) to reference this font.
  pub fn add_font(&mut self, font: ab_glyph::FontArc) -> FontId {
    self.glyph_brush.add_font(font)
  }
}

impl TextRenderer {
  fn process_queued(
    &mut self,
    device: &wgpu::Device,
    staging_belt: &mut wgpu::util::StagingBelt,
    encoder: &mut wgpu::CommandEncoder,
  ) {
    let pipeline = &mut self.pipeline;

    let mut brush_action;

    loop {
      brush_action = self.glyph_brush.process_queued(
        |rect, tex_data| {
          let offset = [rect.min[0] as u16, rect.min[1] as u16];
          let size = [rect.width() as u16, rect.height() as u16];

          pipeline.update_cache(device, staging_belt, encoder, offset, size, tex_data);
        },
        Instance::from_vertex,
      );

      match brush_action {
        Ok(_) => break,
        Err(BrushError::TextureTooSmall { suggested }) => {
          // TODO: Obtain max texture dimensions using `wgpu`
          // This is currently not possible I think. Ask!
          let max_image_dimension = 2048;

          let (new_width, new_height) = if (suggested.0 > max_image_dimension
            || suggested.1 > max_image_dimension)
            && (self.glyph_brush.texture_dimensions().0 < max_image_dimension
              || self.glyph_brush.texture_dimensions().1 < max_image_dimension)
          {
            (max_image_dimension, max_image_dimension)
          } else {
            suggested
          };

          log::warn!(
            "Increasing glyph texture size {old:?} -> {new:?}. \
                             Consider building with `.initial_cache_size({new:?})` to avoid \
                             resizing",
            old = self.glyph_brush.texture_dimensions(),
            new = (new_width, new_height),
          );

          pipeline.increase_cache_size(device, new_width, new_height);
          self.glyph_brush.resize_texture(new_width, new_height);
        }
      }
    }

    match brush_action.unwrap() {
      BrushAction::Draw(verts) => {
        self.pipeline.upload(device, staging_belt, encoder, &verts);
      }
      BrushAction::ReDraw => {}
    };
  }
}

impl TextRenderer {
  pub fn new(
    device: &wgpu::Device,
    filter_mode: wgpu::FilterMode,
    render_format: wgpu::TextureFormat,
    // font_path: &str,
    // raw_builder: glyph_brush::GlyphBrushBuilder<F, H>,
  ) -> Self {
    // Prepare glyph_brush
    let inconsolata = ab_glyph::FontArc::try_from_slice(include_bytes!(
      "C:/Users/mk/Desktop/Inconsolata-Regular.ttf"
    ))
    .unwrap();

    let glyph_brush = GlyphBrushBuilder::using_font(inconsolata).build();

    let (cache_width, cache_height) = glyph_brush.texture_dimensions();
    Self {
      pipeline: Pipeline::new(
        device,
        filter_mode,
        render_format,
        cache_width,
        cache_height,
      ),
      glyph_brush,
    }
  }

  /// Draws all queued sections onto a render target.
  /// See [`queue`](struct.GlyphBrush.html#method.queue).
  ///
  /// It __does not__ submit the encoder command buffer to the device queue.
  ///
  /// Trims the cache, see [caching behaviour](#caching-behaviour).
  ///
  /// # Panics
  /// Panics if the provided `target` has a texture format that does not match
  /// the `render_format` provided on creation of the `GlyphBrush`.
  #[inline]
  pub fn draw_queued(
    &mut self,
    device: &wgpu::Device,
    staging_belt: &mut wgpu::util::StagingBelt,
    encoder: &mut wgpu::CommandEncoder,
    target: &wgpu::TextureView,
    target_width: u32,
    target_height: u32,
  ) -> Result<(), String> {
    self.draw_queued_with_transform(
      device,
      staging_belt,
      encoder,
      target,
      orthographic_projection(target_width, target_height),
    )
  }

  /// Draws all queued sections onto a render target, applying a position
  /// transform (e.g. a projection).
  /// See [`queue`](struct.GlyphBrush.html#method.queue).
  ///
  /// It __does not__ submit the encoder command buffer to the device queue.
  ///
  /// Trims the cache, see [caching behaviour](#caching-behaviour).
  ///
  /// # Panics
  /// Panics if the provided `target` has a texture format that does not match
  /// the `render_format` provided on creation of the `GlyphBrush`.
  #[inline]
  pub fn draw_queued_with_transform(
    &mut self,
    device: &wgpu::Device,
    staging_belt: &mut wgpu::util::StagingBelt,
    encoder: &mut wgpu::CommandEncoder,
    target: &wgpu::TextureView,
    transform: [f32; 16],
  ) -> Result<(), String> {
    self.process_queued(device, staging_belt, encoder);
    self
      .pipeline
      .draw(device, staging_belt, encoder, target, transform, None);

    Ok(())
  }

  /// Draws all queued sections onto a render target, applying a position
  /// transform (e.g. a projection) and a scissoring region.
  /// See [`queue`](struct.GlyphBrush.html#method.queue).
  ///
  /// It __does not__ submit the encoder command buffer to the device queue.
  ///
  /// Trims the cache, see [caching behaviour](#caching-behaviour).
  ///
  /// # Panics
  /// Panics if the provided `target` has a texture format that does not match
  /// the `render_format` provided on creation of the `GlyphBrush`.
  #[inline]
  pub fn draw_queued_with_transform_and_scissoring(
    &mut self,
    device: &wgpu::Device,
    staging_belt: &mut wgpu::util::StagingBelt,
    encoder: &mut wgpu::CommandEncoder,
    target: &wgpu::TextureView,
    transform: [f32; 16],
    region: Region,
  ) -> Result<(), String> {
    self.process_queued(device, staging_belt, encoder);
    self.pipeline.draw(
      device,
      staging_belt,
      encoder,
      target,
      transform,
      Some(region),
    );

    Ok(())
  }
}

/// Helper function to generate a generate a transform matrix.
pub fn orthographic_projection(width: u32, height: u32) -> [f32; 16] {
  #[cfg_attr(rustfmt, rustfmt_skip)]
    [
        2.0 / width as f32, 0.0, 0.0, 0.0,
        0.0, -2.0 / height as f32, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        -1.0, 1.0, 0.0, 1.0,
    ]
}
