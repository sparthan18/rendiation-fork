use crate::*;

pub type ComponentUpdater<C, T> = Box<dyn Fn(&mut C, &T)>;

pub struct ComponentCell<C, T> {
  component: C,
  updater: ComponentUpdater<C, T>,
}

pub trait ComponentCellMaker<T>: Sized {
  fn update_by(self, updater: impl Fn(&mut Self, &T) + 'static) -> ComponentCell<Self, T> {
    ComponentCell {
      component: self,
      updater: Box::new(updater),
    }
  }
}
impl<T, X> ComponentCellMaker<T> for X {}

impl<T, IC, C> ComponentAbility<T, IC> for ComponentCell<C, T>
where
  IC: Component<T>,
  C: ComponentAbility<T, IC>,
{
  fn update(&mut self, model: &T, inner: &mut IC, ctx: &mut UpdateCtx) {
    (self.updater)(&mut self.component, model);
    self.component.update(model, inner, ctx);
  }

  fn event(&mut self, model: &mut T, event: &mut EventCtx, inner: &mut IC) {
    self.component.event(model, event, inner);
  }
}

impl<T, IC: Presentable, C: PresentableAbility<IC>> PresentableAbility<IC> for ComponentCell<C, T> {
  fn render(&mut self, builder: &mut PresentationBuilder, inner: &mut IC) {
    self.component.render(builder, inner);
  }
}

impl<T, C: LayoutAbility<IC>, IC: LayoutAble> LayoutAbility<IC> for ComponentCell<C, T> {
  fn layout(
    &mut self,
    constraint: LayoutConstraint,
    ctx: &mut LayoutCtx,
    inner: &mut IC,
  ) -> LayoutResult {
    self.component.layout(constraint, ctx, inner)
  }

  fn set_position(&mut self, position: UIPosition, inner: &mut IC) {
    self.component.set_position(position, inner);
  }
}

impl<T, C: HotAreaPassBehavior<IC>, IC> HotAreaPassBehavior<IC> for ComponentCell<C, T> {
  fn is_point_in(&self, point: crate::UIPosition, inner: &IC) -> bool {
    self.component.is_point_in(point, inner)
  }
}

impl<C, T> Component<T> for ComponentCell<C, T> {
  fn event(&mut self, _model: &mut T, _event: &mut EventCtx<'_>) {}

  fn update(&mut self, model: &T, ctx: &mut UpdateCtx) {
    (self.updater)(&mut self.component, model)
  }
}
