#![feature(specialization)]
#![feature(const_generics)]
#![feature(trait_alias)]
#![feature(stmt_expr_attributes)]
#![allow(incomplete_features)]
#![allow(clippy::many_single_char_names)]
#![allow(clippy::transmute_ptr_to_ptr)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

pub mod arithmetic;
pub mod consts;
pub mod interpolation;
pub mod mat;
pub mod quat;
pub mod utils;
pub mod vec;

pub mod wasm;
pub use wasm::*;

pub use arithmetic::*;
pub use interpolation::*;
pub use mat::*;
pub use vec::*;

pub use self::consts::*;
pub use self::quat::*;

pub use num_traits::*;

#[macro_use]
pub mod marcos;

pub trait Scalar = Float + Half + Three + Two;

pub trait SpaceEntity<T: Scalar, const D: usize> {
  fn apply_matrix(&mut self, mat: SquareMatrixType<T, D>) -> &mut Self;
}
