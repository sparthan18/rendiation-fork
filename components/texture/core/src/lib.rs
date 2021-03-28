pub mod address;
use std::{
  ops::{Deref, DerefMut},
  path::Path,
};

pub use address::*;
pub mod filter;
pub use filter::*;
pub mod cube;
pub use cube::*;
pub mod sampler;
pub use sampler::*;

use image::ImageBuffer;
use rendiation_algebra::Vec2;

pub use image::*;

pub trait Texture2D {
  type Pixel;
  fn get(&self, position: Vec2<usize>) -> &Self::Pixel;
  fn get_mut(&mut self, position: Vec2<usize>) -> &mut Self::Pixel;

  fn write(&mut self, position: Vec2<usize>, v: Self::Pixel) {
    *self.get_mut(position) = v;
  }

  fn size(&self) -> (usize, usize);

  fn pixel_count(&self) -> usize {
    let (width, height) = self.size();
    width * height
  }

  fn save_to_file<P: AsRef<Path>>(&self, path: P);
}

impl<P, C> Texture2D for ImageBuffer<P, C>
where
  P: Pixel + 'static,
  [P::Subpixel]: EncodableLayout,
  C: Deref<Target = [P::Subpixel]>,
  C: DerefMut<Target = [P::Subpixel]>,
{
  type Pixel = P;

  fn get(&self, position: Vec2<usize>) -> &Self::Pixel {
    self.get_pixel(position.x as u32, position.y as u32)
  }

  fn get_mut(&mut self, position: Vec2<usize>) -> &mut Self::Pixel {
    self.get_pixel_mut(position.x as u32, position.y as u32)
  }

  fn size(&self) -> (usize, usize) {
    let d = self.dimensions();
    (d.0 as usize, d.1 as usize)
  }
  fn save_to_file<Pa: AsRef<Path>>(&self, path: Pa) {
    self.save(path).unwrap();
  }
}
