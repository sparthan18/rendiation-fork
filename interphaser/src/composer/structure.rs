use crate::*;

pub struct If<T, C> {
  should_render: Box<dyn Fn(&T) -> bool>,
  func: Box<dyn Fn(&T) -> C>,
  inner: Option<C>,
}

impl<T, C> If<T, C>
where
  C: Component<T>,
{
  pub fn condition<F, SF>(should_render: SF, func: F) -> Self
  where
    SF: Fn(&T) -> bool + 'static,
    F: Fn(&T) -> C + 'static,
  {
    Self {
      should_render: Box::new(should_render),
      func: Box::new(func),
      inner: None,
    }
  }

  pub fn else_condition<F, EC>(self, func: F) -> Else<T, C, EC>
  where
    F: Fn(&T) -> EC + 'static,
  {
    Else {
      if_com: self,
      func: Box::new(func),
      inner: None,
    }
  }
}

impl<T, C> Component<T> for If<T, C>
where
  C: Component<T>,
{
  fn update(&mut self, model: &T, ctx: &mut UpdateCtx) {
    if (self.should_render)(model) {
      if let Some(inner) = &mut self.inner {
        inner.update(model, ctx);
      } else {
        self.inner = Some((self.func)(model));
      }
    } else {
      self.inner = None;
    }
  }

  fn event(&mut self, model: &mut T, event: &mut crate::EventCtx) {
    if let Some(inner) = &mut self.inner {
      inner.event(model, event)
    }
  }
}

impl<T, C: LayoutAble> LayoutAble for If<T, C> {
  fn layout(&mut self, constraint: LayoutConstraint, ctx: &mut LayoutCtx) -> LayoutResult {
    if let Some(inner) = &mut self.inner {
      inner.layout(constraint, ctx)
    } else {
      LayoutResult {
        size: constraint.min(),
        baseline_offset: 0.,
      }
    }
  }

  fn set_position(&mut self, position: UIPosition) {
    if let Some(inner) = &mut self.inner {
      inner.set_position(position)
    }
  }
}

impl<T, C: Presentable> Presentable for If<T, C> {
  fn render(&mut self, builder: &mut PresentationBuilder) {
    if let Some(inner) = &mut self.inner {
      inner.render(builder)
    }
  }
}

pub struct Else<T, C, EC> {
  if_com: If<T, C>,
  func: Box<dyn Fn(&T) -> EC>,
  inner: Option<EC>,
}

impl<T, C, EC> Component<T> for Else<T, C, EC>
where
  C: Component<T>,
  EC: Component<T>,
{
  fn update(&mut self, model: &T, ctx: &mut UpdateCtx) {
    self.if_com.update(model, ctx);

    if self.if_com.inner.is_none() {
      if let Some(inner) = &mut self.inner {
        inner.update(model, ctx);
      } else {
        self.inner = Some((self.func)(model));
      }
    } else {
      self.inner = None
    }
  }

  fn event(&mut self, model: &mut T, event: &mut crate::EventCtx) {
    if let Some(inner) = &mut self.inner {
      inner.event(model, event)
    } else {
      self.if_com.event(model, event);
    }
  }
}

impl<T, C: LayoutAble, EC: LayoutAble> LayoutAble for Else<T, C, EC> {
  fn layout(&mut self, constraint: LayoutConstraint, ctx: &mut LayoutCtx) -> LayoutResult {
    if let Some(inner) = &mut self.inner {
      inner.layout(constraint, ctx)
    } else {
      self.if_com.layout(constraint, ctx)
    }
  }

  fn set_position(&mut self, position: UIPosition) {
    if let Some(inner) = &mut self.inner {
      inner.set_position(position)
    } else {
      self.if_com.set_position(position);
    }
  }
}

impl<T, C: Presentable, EC: Presentable> Presentable for Else<T, C, EC> {
  fn render(&mut self, builder: &mut PresentationBuilder) {
    if let Some(inner) = &mut self.inner {
      inner.render(builder)
    } else {
      self.if_com.render(builder);
    }
  }
}

/// if item's key not changed, we consider this item should update not destroy
pub trait IdentityKeyed {
  type Key: PartialEq;
  fn key(&self) -> Self::Key;
}

pub struct For<T: IdentityKeyed, C> {
  children: Vec<(T::Key, C)>,
  mapper: Box<dyn Fn(usize) -> C>,
}

impl<T, C> For<T, C>
where
  T: IdentityKeyed,
  C: Component<T>,
{
  pub fn by<F>(mapper: F) -> Self
  where
    F: Fn(usize) -> C + 'static,
  {
    Self {
      children: Vec::new(),
      mapper: Box::new(mapper),
    }
  }
}

impl<'a, T, C> Component<Vec<T>> for For<T, C>
where
  T: 'static + IdentityKeyed + Clone,
  C: Component<T>,
{
  fn update(&mut self, model: &Vec<T>, ctx: &mut UpdateCtx) {
    // todo should optimize
    self.children = model
      .iter()
      .enumerate()
      .map(|(index, item)| {
        let new_key = item.key();

        if let Some(previous) = self.children.iter().position(|cached| cached.0 == new_key) {
          // move
          self.children.swap_remove(previous)
        } else {
          // new
          (new_key, (self.mapper)(index))
        }
      })
      .collect();
    // and not exist will be drop

    self
      .children
      .iter_mut()
      .zip(model)
      .for_each(|((_, c), m)| c.update(m, ctx))
  }

  fn event(&mut self, model: &mut Vec<T>, event: &mut crate::EventCtx) {
    self
      .children
      .iter_mut()
      .zip(model)
      .for_each(|((_, item), model)| item.event(model, event))
  }
}

type IterType<'a, C: 'static, T: 'static + IdentityKeyed> =
  impl Iterator<Item = &'a mut C> + 'a + ExactSizeIterator;

impl<'a, T: 'static + IdentityKeyed, C: 'static> IntoIterator for &'a mut For<T, C> {
  type Item = &'a mut C;
  type IntoIter = IterType<'a, C, T>;

  fn into_iter(self) -> IterType<'a, C, T> {
    self.children.iter_mut().map(|(_, c)| c)
  }
}

impl<T: IdentityKeyed, C: Presentable> Presentable for For<T, C> {
  fn render(&mut self, builder: &mut PresentationBuilder) {
    self
      .children
      .iter_mut()
      .for_each(|(_, c)| c.render(builder))
  }
}

#[derive(Default)]
pub struct ComponentArray<C> {
  pub children: Vec<C>,
}

impl<X> ComponentArray<X> {
  pub fn child(mut self, x: X) -> Self {
    self.children.push(x);
    self
  }
}

type IterType2<'a, C: 'static> = impl Iterator<Item = &'a mut C> + 'a + ExactSizeIterator;

impl<'a, C: 'static> IntoIterator for &'a mut ComponentArray<C> {
  type Item = &'a mut C;
  type IntoIter = IterType2<'a, C>;

  fn into_iter(self) -> IterType2<'a, C> {
    self.children.iter_mut()
  }
}

impl<C: Presentable> Presentable for ComponentArray<C> {
  fn render(&mut self, builder: &mut PresentationBuilder) {
    self.children.iter_mut().for_each(|c| c.render(builder))
  }
}

impl<'a, T, C> Component<T> for ComponentArray<C>
where
  C: Component<T>,
{
  fn update(&mut self, model: &T, ctx: &mut UpdateCtx) {
    self.children.iter_mut().for_each(|c| c.update(model, ctx))
  }

  fn event(&mut self, model: &mut T, event: &mut crate::EventCtx) {
    self.children.iter_mut().for_each(|c| c.event(model, event))
  }
}