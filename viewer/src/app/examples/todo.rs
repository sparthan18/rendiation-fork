use interphaser::*;

use crate::button;

pub struct Todo {
  pub items: Vec<TodoItem>,
}

#[derive(Clone, PartialEq)]
pub struct TodoItem {
  pub id: usize,
  pub name: String,
}

// todo change to id
impl IdentityKeyed for TodoItem {
  type Key = usize;

  fn key(&self) -> Self::Key {
    self.id
  }
}

pub fn build_todo() -> impl UIComponent<Todo> {
  For::by(|_| Child::flex(build_todo_item(), 1.))
    .extend(Flex::column())
    .extend(TodoItemDeleteHandler::by(|s: &mut Vec<TodoItem>, _, e| {
      s.remove(s.iter().position(|item| item.name == e.name).unwrap());
    }))
    .extend(Container::size((800., 1000.)))
    .lens(lens!(Todo, items))
}

pub struct TodoItemDeleteEvent {
  name: String,
}

#[derive(Default)]
pub struct TodoItemDelete;
pub type TodoItemDeleteHandler<T> = EventHandler<T, TodoItemDelete>;
impl EventHandlerType for TodoItemDelete {
  type Event = TodoItemDeleteEvent;
}
impl<C> EventHandlerImpl<C> for TodoItemDelete {
  fn downcast_event<'a>(&mut self, event: &'a mut EventCtx, _inner: &C) -> Option<&'a Self::Event> {
    event
      .custom_event
      .consume_if_type_is::<TodoItemDeleteEvent>()
  }
  fn should_handle_in_bubble(&self) -> bool {
    true
  }
}

pub fn build_todo_item() -> impl UIComponent<TodoItem> {
  let label = Text::default()
    .editable()
    .lens(lens!(TodoItem, name))
    .extend(Container::size((200., 100.)));

  let button = button("delete", |s: &mut TodoItem, c, _| {
    println!("delete {}", s.name);
    c.emit(TodoItemDeleteEvent {
      name: s.name.clone(),
    })
  });

  flex_group()
    .child(Child::flex(label, 1.))
    .child(Child::flex(button, 1.))
    .extend(Flex::row())
    .extend(Container::size((500., 120.)))
}