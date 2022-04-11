#![feature(explicit_generic_args_with_impl_trait)]
#![feature(specialization)]
#![allow(incomplete_features)]

pub mod code_gen;
pub use code_gen::*;

pub mod api;
pub mod gir;
pub mod graph;
pub mod link;

pub use api::*;
pub use gir::*;
pub use graph::*;
pub use link::*;

pub mod traits_impl;
pub use traits_impl::*;

pub mod std140;
pub use std140::*;

pub use bytemuck::*;
pub use memoffset::offset_of;
pub use rendiation_algebra::*;
pub use shader_derives::*;

use arena_graph::*;
use std::{
  any::{Any, TypeId},
  cell::{Cell, RefCell, UnsafeCell},
  collections::HashMap,
  collections::HashSet,
  hash::{Hash, Hasher},
  marker::PhantomData,
  ops::*,
  rc::Rc,
  sync::atomic::{AtomicUsize, Ordering},
};

#[cfg(test)]
mod test;