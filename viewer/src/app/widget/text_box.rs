use interphaser::*;

pub fn text_box(
  placeholder: impl Into<String> + 'static + Copy,
  on_change: impl Fn(&mut String) + 'static + Copy,
) -> impl UIComponent<String> {
  If::condition(
    |t: &String| t.is_empty(),
    move |_t| {
      Text::default()
        .bind(move |s, _| s.content.set(placeholder))
        .extend(Container::size((200., 80.)))
    },
  )
  .else_condition(move |_| Text::default().editable().on_change(on_change))
}