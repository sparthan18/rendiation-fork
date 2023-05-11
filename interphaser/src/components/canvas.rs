use std::rc::Rc;

use rendiation_texture::Size;
use webgpu::{map_size_gpu, GPU2DTextureView, GPUTexture, GPU};
use winit::event::Event;

use crate::*;

pub struct GPUCanvas {
  current_render_buffer_size: Size,
  content: Option<GPU2DTextureView>,
  layout: LayoutUnit,
}

impl Default for GPUCanvas {
  fn default() -> Self {
    Self {
      current_render_buffer_size: Size::from_u32_pair_min_one((100, 100)),
      content: None,
      layout: Default::default(),
    }
  }
}

impl Presentable for GPUCanvas {
  fn render(&mut self, builder: &mut PresentationBuilder) {
    self.layout.update_world(builder.current_origin_offset());
    if let Some(content) = &self.content {
      builder.present.primitives.push(Primitive::Quad((
        self.layout.into_quad(),
        Style::Texture(content.clone()),
      )));
    }
  }
}

impl LayoutAble for GPUCanvas {
  fn layout(&mut self, constraint: LayoutConstraint, _ctx: &mut LayoutCtx) -> LayoutResult {
    self.layout.size = constraint.max();
    self.layout.size.with_default_baseline()
  }

  fn set_position(&mut self, position: UIPosition) {
    self.layout.set_relative_position(position)
  }
}

pub struct CanvasWindowPositionInfo {
  /// in window coordinates
  pub absolute_position: UIPosition,
  pub size: UISize,
}

impl CanvasWindowPositionInfo {
  pub fn compute_normalized_position_in_canvas_coordinate(
    &self,
    states: &WindowState,
  ) -> (f32, f32) {
    let canvas_x = states.mouse_position.x - self.absolute_position.x;
    let canvas_y = states.mouse_position.y - self.absolute_position.y;

    (
      canvas_x / self.size.width * 2. - 1.,
      -(canvas_y / self.size.height * 2. - 1.),
    )
  }
}

pub trait CanvasPrinter {
  fn event(
    &mut self,
    event: &winit::event::Event<()>,
    states: &WindowState,
    position_info: CanvasWindowPositionInfo,
  );
  fn update_render_size(&mut self, layout_size: (f32, f32)) -> Size;
  fn draw_canvas(&mut self, gpu: &Rc<GPU>, canvas: GPU2DTextureView);
}

impl<T: CanvasPrinter> Component<T> for GPUCanvas {
  fn event(&mut self, model: &mut T, event: &mut EventCtx) {
    let position_info = CanvasWindowPositionInfo {
      absolute_position: self.layout.absolute_position,
      size: self.layout.size,
    };

    model.event(event.event, event.states, position_info);
    match event.event {
      Event::RedrawRequested(_) => {
        let new_size = model.update_render_size(self.layout.size.into());
        if new_size != self.current_render_buffer_size {
          self.content = None;
        }

        let target = self.content.get_or_insert_with(|| {
          let device = &event.gpu.device;

          let desc = webgpu::TextureDescriptor {
            label: "interphase-canvas-output".into(),
            size: map_size_gpu(new_size),
            dimension: webgpu::TextureDimension::D2,
            format: webgpu::TextureFormat::Rgba8Unorm,
            view_formats: &[],
            usage: webgpu::TextureUsages::TEXTURE_BINDING
              | webgpu::TextureUsages::COPY_DST
              | webgpu::TextureUsages::COPY_SRC
              | webgpu::TextureUsages::RENDER_ATTACHMENT,
            mip_level_count: 1,
            sample_count: 1,
          };

          let texture = GPUTexture::create(desc, device);
          texture.create_view(Default::default()).try_into().unwrap()
        });

        model.draw_canvas(&event.gpu, target.clone());
      }
      _ => {}
    }
  }
}
