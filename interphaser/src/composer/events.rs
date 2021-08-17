use std::any::Any;

use crate::*;
use winit::event::*;

pub struct EventHandleCtx {
  custom_event_emitter: CustomEventEmitter,
}

impl EventHandleCtx {
  pub fn emit(&mut self, e: impl Any) {
    self.custom_event_emitter.emit(e)
  }
}

pub trait HotAreaProvider {
  fn is_point_in(&self, point: UIPosition) -> bool;
}

pub struct EventHandler<T, X: EventHandlerType> {
  state: X,
  handler: Box<dyn Fn(&mut T, &mut EventHandleCtx, &X::Event)>,
}

impl<T, X: EventHandlerType> EventHandler<T, X> {
  pub fn by(fun: impl Fn(&mut T, &mut EventHandleCtx, &X::Event) + 'static) -> Self {
    Self {
      state: Default::default(),
      handler: Box::new(fun),
    }
  }

  fn handle_event<C>(&mut self, model: &mut T, event: &mut EventCtx, inner: &mut C)
  where
    X: EventHandlerImpl<C>,
  {
    event.custom_event.update();
    if let Some(e) = self.state.downcast_event(event, inner) {
      let mut ctx = EventHandleCtx {
        custom_event_emitter: Default::default(),
      };
      (self.handler)(model, &mut ctx, e);
      event.custom_event.merge(ctx.custom_event_emitter);
    }
  }
}

impl<T, X: EventHandlerImpl<C>, C: Component<T>> ComponentAbility<T, C> for EventHandler<T, X> {
  fn event(&mut self, model: &mut T, event: &mut EventCtx, inner: &mut C) {
    if !self.state.should_handle_in_bubble() {
      self.handle_event(model, event, inner);
    }

    inner.event(model, event);

    if self.state.should_handle_in_bubble() {
      self.handle_event(model, event, inner);
    }
  }

  fn update(&mut self, model: &T, inner: &mut C, ctx: &mut UpdateCtx) {
    inner.update(model, ctx);
  }
}

impl<T, X: EventHandlerType, C: Presentable> PresentableAbility<C> for EventHandler<T, X> {
  fn render(&mut self, builder: &mut PresentationBuilder, inner: &mut C) {
    inner.render(builder);
  }
}

impl<T, X: EventHandlerType, C: LayoutAble> LayoutAbility<C> for EventHandler<T, X> {
  fn layout(
    &mut self,
    constraint: LayoutConstraint,
    ctx: &mut LayoutCtx,
    inner: &mut C,
  ) -> LayoutResult {
    inner.layout(constraint, ctx)
  }

  fn set_position(&mut self, position: UIPosition, inner: &mut C) {
    inner.set_position(position)
  }
}

impl<T, X: EventHandlerType, C: HotAreaProvider> HotAreaPassBehavior<C> for EventHandler<T, X> {
  fn is_point_in(&self, point: crate::UIPosition, inner: &C) -> bool {
    inner.is_point_in(point)
  }
}

pub trait EventHandlerType: Default {
  type Event;
}

pub trait EventHandlerImpl<C>: EventHandlerType {
  fn downcast_event<'a>(&mut self, event: &'a mut EventCtx, inner: &C) -> Option<&'a Self::Event>;
  fn should_handle_in_bubble(&self) -> bool {
    false
  }
}

#[derive(Default)]
pub struct MouseDown;
pub type MouseDownHandler<T> = EventHandler<T, MouseDown>;
impl EventHandlerType for MouseDown {
  type Event = ();
}
impl<C: HotAreaProvider> EventHandlerImpl<C> for MouseDown {
  fn downcast_event<'a>(&mut self, event: &'a mut EventCtx, inner: &C) -> Option<&'a Self::Event> {
    if let Some((MouseButton::Left, ElementState::Pressed)) = mouse(event.event) {
      if inner.is_point_in(event.states.mouse_position) {
        return Some(&());
      }
    }
    None
  }
}

#[derive(Default)]
pub struct MouseUp;
pub type MouseUpHandler<T> = EventHandler<T, MouseUp>;
impl EventHandlerType for MouseUp {
  type Event = ();
}
impl<C: HotAreaProvider> EventHandlerImpl<C> for MouseUp {
  fn downcast_event<'a>(&mut self, event: &'a mut EventCtx, inner: &C) -> Option<&'a Self::Event> {
    if let Some((MouseButton::Left, ElementState::Released)) = mouse(event.event) {
      if inner.is_point_in(event.states.mouse_position) {
        return Some(&());
      }
    }
    None
  }
}

pub struct Click {
  mouse_down: bool,
}
impl Default for Click {
  fn default() -> Self {
    Self { mouse_down: false }
  }
}

pub type ClickHandler<T> = EventHandler<T, Click>;
impl EventHandlerType for Click {
  type Event = ();
}
impl<C: HotAreaProvider> EventHandlerImpl<C> for Click {
  fn downcast_event<'a>(&mut self, event: &'a mut EventCtx, inner: &C) -> Option<&'a Self::Event> {
    if let Some((MouseButton::Left, ElementState::Pressed)) = mouse(event.event) {
      if inner.is_point_in(event.states.mouse_position) {
        self.mouse_down = true;
      }
    } else if let Some((MouseButton::Left, ElementState::Released)) = mouse(event.event) {
      if self.mouse_down && inner.is_point_in(event.states.mouse_position) {
        return Some(&());
      }
      self.mouse_down = false;
    }
    None
  }
}

#[derive(Default)]
pub struct MouseMove;
pub type MouseMoveHandler<T> = EventHandler<T, MouseMove>;
impl EventHandlerType for MouseMove {
  type Event = ();
}
impl<C: HotAreaProvider> EventHandlerImpl<C> for MouseMove {
  fn downcast_event<'a>(&mut self, event: &'a mut EventCtx, inner: &C) -> Option<&'a Self::Event> {
    if let Some(position) = mouse_move(event.event) {
      if inner.is_point_in((position.x as f32, position.y as f32).into()) {
        return Some(&());
      }
    }
    None
  }
}

pub struct MouseIn {
  is_mouse_in: bool,
}
impl Default for MouseIn {
  fn default() -> Self {
    Self { is_mouse_in: false }
  }
}
pub type MouseInHandler<T> = EventHandler<T, MouseIn>;
impl EventHandlerType for MouseIn {
  type Event = ();
}
impl<C: HotAreaProvider> EventHandlerImpl<C> for MouseIn {
  fn downcast_event<'a>(&mut self, event: &'a mut EventCtx, inner: &C) -> Option<&'a Self::Event> {
    if let Some(position) = mouse_move(event.event) {
      if inner.is_point_in((position.x as f32, position.y as f32).into()) {
        if !self.is_mouse_in {
          self.is_mouse_in = true;
          return Some(&());
        }
        self.is_mouse_in = true;
      } else {
        self.is_mouse_in = false;
      }
    }
    None
  }
}

pub struct MouseOut {
  is_mouse_in: bool,
}
impl Default for MouseOut {
  fn default() -> Self {
    Self { is_mouse_in: false }
  }
}
pub type MouseOutHandler<T> = EventHandler<T, MouseOut>;
impl EventHandlerType for MouseOut {
  type Event = ();
}
impl<C: HotAreaProvider> EventHandlerImpl<C> for MouseOut {
  fn downcast_event<'a>(&mut self, event: &'a mut EventCtx, inner: &C) -> Option<&'a Self::Event> {
    if let Some(position) = mouse_move(event.event) {
      if !inner.is_point_in((position.x as f32, position.y as f32).into()) {
        if self.is_mouse_in {
          self.is_mouse_in = false;
          return Some(&());
        }
        self.is_mouse_in = false;
      } else {
        self.is_mouse_in = true;
      }
    }
    None
  }
}

fn window_event<'a>(event: &'a Event<()>) -> Option<&'a WindowEvent<'a>> {
  match event {
    Event::WindowEvent { event, .. } => Some(event),
    _ => None,
  }
}

fn mouse(event: &Event<()>) -> Option<(MouseButton, ElementState)> {
  window_event(event).and_then(|e| match e {
    WindowEvent::MouseInput { state, button, .. } => Some((*button, *state)),
    _ => None,
  })
}

fn mouse_move(event: &Event<()>) -> Option<winit::dpi::PhysicalPosition<f64>> {
  window_event(event).and_then(|e| match e {
    WindowEvent::CursorMoved { position, .. } => Some(*position),
    _ => None,
  })
}
