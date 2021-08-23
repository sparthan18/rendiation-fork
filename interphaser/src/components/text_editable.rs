use glyph_brush::ab_glyph::Font;

use crate::*;

pub struct EditableText {
  text: Text,
  cursor: Option<Cursor>,
}

use std::ops::{Deref, DerefMut};
impl Deref for EditableText {
  type Target = Text;

  fn deref(&self) -> &Self::Target {
    &self.text
  }
}

impl DerefMut for EditableText {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.text
  }
}

impl EditableText {
  fn update_cursor_by_click(&mut self, position: UIPosition, fonts: &FontManager) {
    let layout = self.text.get_text_layout(fonts);
    let rect = layout
      .iter()
      .map(|sg| fonts.get_font(sg.font_id).glyph_bounds(&sg.glyph))
      .enumerate()
      .find(|(_, rect)| {
        position.x >= rect.min.x
          && position.x <= rect.max.x
          && position.y >= rect.min.y
          && position.y <= rect.max.y
      });

    if let Some((index, rect)) = rect {
      let height = rect.max.y - rect.min.y;
      let (text_index, position) = if position.x >= (rect.max.x + rect.min.x) / 2. {
        (index + 1, (rect.max.x, rect.min.y))
      } else {
        (index, (rect.min.x, rect.min.y))
      };

      self.cursor = Cursor {
        position: position.into(),
        height,
        text_index,
      }
      .into()
    }
  }

  fn update_cursor_position(&mut self, fonts: &FontManager) {
    if let Some(cursor) = &mut self.cursor {
      let layout = self.text.get_text_layout(fonts);
      let index = if cursor.text_index ==0 {
        0
      } else {
        cursor.text_index - 1
      };
      let sg = &layout[index];
      let rect = fonts.get_font(sg.font_id).glyph_bounds(&sg.glyph);

      let height = rect.max.y - rect.min.y;
      let position = if cursor.text_index == 0 {
        (rect.min.x, rect.min.y)
      } else {
        (rect.max.x, rect.min.y)
      };
      cursor.position = position.into();
      cursor.height = height;
    }
  }

  fn insert_at_cursor(&mut self, c: char, model: &mut String, fonts: &FontManager) {
    if let Some(cursor) = &mut self.cursor {
      let index = cursor.text_index;
      model.insert(index, c);

      self.text.content.set(model.clone());
      self.text.reset_text_layout();
      cursor.text_index += 1;
    }
    self.update_cursor_position(fonts)
  }

  fn delete_at_cursor(&mut self, model: &mut String, fonts: &FontManager) {
    if let Some(cursor) = &mut self.cursor {
      if cursor.text_index == 0 {
        return;
      }
      model.remove(cursor.text_index - 1);

      self.text.content.set(model.clone());
      self.text.reset_text_layout();
      cursor.text_index -= 1;
    }
    self.update_cursor_position(fonts)
  }

  fn input(&mut self, key: winit::event::VirtualKeyCode, model: &mut String, fonts: &FontManager) {
    use winit::event::VirtualKeyCode::*;
    let input = match key {
      Key1 => '1'.into(),
      Key2 => '2'.into(),
      Key3 => '3'.into(),
      Key4 => '4'.into(),
      Key5 => '5'.into(),
      Key6 => '6'.into(),
      Key7 => '7'.into(),
      Key8 => '8'.into(),
      Key9 => '9'.into(),
      Key0 => '0'.into(),
      A => 'a'.into(),
      B => 'b'.into(),
      C => 'c'.into(),
      D => 'd'.into(),
      E => 'e'.into(),
      F => 'f'.into(),
      G => 'g'.into(),
      H => 'h'.into(),
      I => 'i'.into(),
      J => 'j'.into(),
      K => 'k'.into(),
      L => 'l'.into(),
      M => 'm'.into(),
      N => 'n'.into(),
      O => 'o'.into(),
      P => 'p'.into(),
      Q => 'q'.into(),
      R => 'r'.into(),
      S => 's'.into(),
      T => 't'.into(),
      U => 'u'.into(),
      V => 'v'.into(),
      W => 'w'.into(),
      X => 'x'.into(),
      Y => 'y'.into(),
      Z => 'z'.into(),
      _ => None,
    };

    if let Some(c) = input {
      self.insert_at_cursor(c, model, fonts);
    }

    match key {
      // Escape => todo!(),
      // Left => todo!(),
      // Up => todo!(),
      // Right => todo!(),
      // Down => todo!(),
      Back => {
        self.delete_at_cursor(model, fonts);
      }
      // Return => todo!(),
      // Space => todo!(),
      _ => {}
    }
  }
}

impl Text {
  pub fn editable(self) -> EditableText {
    EditableText {
      text: self,
      cursor: None,
    }
  }
}

pub struct Cursor {
  // top_start
  position: UIPosition,
  height: f32,
  text_index: usize,
}

impl Cursor {
  pub fn create_quad(&self) -> Quad {
    Quad {
      x: self.position.x,
      y: self.position.y,
      width: 1.,
      height: self.height,
    }
  }
}

impl Component<String> for EditableText {
  fn event(&mut self, model: &mut String, ctx: &mut EventCtx) {
    self.text.event(model, ctx);

    use winit::event::*;

    match ctx.event {
      Event::WindowEvent { event, .. } => match event {
        WindowEvent::KeyboardInput { input, .. } => {
          if let Some(virtual_keycode) = input.virtual_keycode {
            if input.state == ElementState::Pressed {
              self.input(virtual_keycode, model, ctx.fonts);
            }
          }
        }
        WindowEvent::MouseInput { state, button, .. } => {
          if let (MouseButton::Left, ElementState::Pressed) = (button, state) {
            self.update_cursor_by_click(ctx.states.mouse_position, &ctx.fonts)
          }
        }
        _ => {}
      },
      _ => {}
    }
  }

  fn update(&mut self, model: &String, ctx: &mut UpdateCtx) {
    self.text.content.set(model);
    self.text.update(model, ctx)
  }
}

impl Presentable for EditableText {
  fn render(&mut self, builder: &mut PresentationBuilder) {
    self.text.render(builder);
    if let Some(cursor) = &self.cursor {
      builder.present.primitives.push(Primitive::Quad((
        cursor.create_quad(),
        Style::SolidColor((0., 0., 0., 1.).into()),
      )));
    }
  }
}

impl LayoutAble for EditableText {
  fn layout(&mut self, constraint: LayoutConstraint, ctx: &mut LayoutCtx) -> LayoutResult {
    self.text.layout(constraint, ctx)
  }

  fn set_position(&mut self, position: UIPosition) {
    self.text.set_position(position)
  }
}
