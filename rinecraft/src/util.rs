use crate::image_data::ImageData;
use crate::vertex::*;

pub fn create_vertices() -> (Vec<Vertex>, Vec<u16>) {
  let vertex_data = [
    // top (0, 0, 1)
    vertex([-1, -1, 1], [0, 0]),
    vertex([1, -1, 1], [1, 0]),
    vertex([1, 1, 1], [1, 1]),
    vertex([-1, 1, 1], [0, 1]),
    // bottom (0, 0, -1)
    vertex([-1, 1, -1], [1, 0]),
    vertex([1, 1, -1], [0, 0]),
    vertex([1, -1, -1], [0, 1]),
    vertex([-1, -1, -1], [1, 1]),
    // right (1, 0, 0)
    vertex([1, -1, -1], [0, 0]),
    vertex([1, 1, -1], [1, 0]),
    vertex([1, 1, 1], [1, 1]),
    vertex([1, -1, 1], [0, 1]),
    // left (-1, 0, 0)
    vertex([-1, -1, 1], [1, 0]),
    vertex([-1, 1, 1], [0, 0]),
    vertex([-1, 1, -1], [0, 1]),
    vertex([-1, -1, -1], [1, 1]),
    // front (0, 1, 0)
    vertex([1, 1, -1], [1, 0]),
    vertex([-1, 1, -1], [0, 0]),
    vertex([-1, 1, 1], [0, 1]),
    vertex([1, 1, 1], [1, 1]),
    // back (0, -1, 0)
    vertex([1, -1, 1], [0, 0]),
    vertex([-1, -1, 1], [1, 0]),
    vertex([-1, -1, -1], [1, 1]),
    vertex([1, -1, -1], [0, 1]),
  ];

  let index_data: &[u16] = &[
    0, 1, 2, 2, 3, 0, // top
    4, 5, 6, 6, 7, 4, // bottom
    8, 9, 10, 10, 11, 8, // right
    12, 13, 14, 14, 15, 12, // left
    16, 17, 18, 18, 19, 16, // front
    20, 21, 22, 22, 23, 20, // back
  ];

  (vertex_data.to_vec(), index_data.to_vec())
}

pub fn create_texels(size: usize) -> ImageData {
  use std::iter;

  let data = (0..size * size)
    .flat_map(|id| {
      // get high five for recognizing this ;)
      let cx = 3.0 * (id % size) as f32 / (size - 1) as f32 - 2.0;
      let cy = 2.0 * (id / size) as f32 / (size - 1) as f32 - 1.0;
      let (mut x, mut y, mut count) = (cx, cy, 0);
      while count < 0xFF && x * x + y * y < 4.0 {
        let old_x = x;
        x = x * x - y * y + cx;
        y = 2.0 * old_x * y + cy;
        count += 1;
      }
      iter::once(0xFF - (count * 5) as u8)
        .chain(iter::once(0xFF - (count * 15) as u8))
        .chain(iter::once(0xFF - (count * 50) as u8))
        .chain(iter::once(1))
    })
    .collect();

  ImageData {
    data,
    width: size,
    height: size,
  }
}

pub trait IndexedItem{
  fn get_index(&self)-> Option<usize>;
  fn update_index(&self);
}

pub struct ArrayContainer<T: IndexedItem>{
  data: Vec<Option<T>>,
  tomb_list: Vec<usize>
}

impl<T: IndexedItem> ArrayContainer<T>{
  pub fn new() -> ArrayContainer<T>{
    ArrayContainer{
      data: Vec::new(),
      tomb_list: Vec::new(),
    }
  }

  pub fn get_mut(&mut self, index: usize) -> &mut T {
    if let Some(data) = &mut self.data[index] {
      data
    }else{
      panic!("try get a deleted item in array container")
    }
  }


  pub fn get(&self, index: usize) -> &T {
    if let Some(data) = &self.data[index] {
      data
    }else{
      panic!("try get a deleted item in array container")
    }
  }

  pub fn set_item(&mut self, item: T){
    if item.get_index().is_some() {
      panic!("has stored before")
    } else{
      let free_index = self.get_free_index();
      if free_index >= self.data.len() {
        self.data.push(Some(item));
      }else{
        self.data[free_index] = Some(item);
      }
    }
  }

  fn get_free_index(&mut self) -> usize{
    let free_index;
    if let Some(i) = self.tomb_list.pop() {
      free_index = i;
    } else {
      free_index = self.data.len();
    }
    free_index
  }

  pub fn delete_item(&mut self, index: usize){
    self.data[index] = None;
    self.tomb_list.push(index);
  }
}