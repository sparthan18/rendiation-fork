use std::marker::PhantomData;

mod example;

pub trait Component<T> {
  fn event(&mut self, state: &mut T) {}

  fn update(&mut self, model: &T) {}
}

pub enum ValueCell<T, U> {
  Static(T),
  Dynamic(DynamicValue<T, U>),
}
impl<T, U> ValueCell<T, U> {
  pub fn update(&mut self, ctx: &U) {
    todo!()
  }
}

pub struct DynamicValue<T, U> {
  fun: Box<dyn Fn(&U) -> T>,
  value: T,
}

pub struct Text<T> {
  content: ValueCell<String, T>,
}

impl<T> Component<T> for Text<T> {
  fn update(&mut self, model: &T) {
    self.content.update(model);
  }
}

pub struct ClickArea<T, C> {
  inner: C,
  phantom: PhantomData<T>,
}

pub struct Container<T, C> {
  width: f32,
  height: f32,
  inner: C,
  phantom: PhantomData<T>,
}

fn button<T>() -> impl Component<T> {
  todo!()
}
