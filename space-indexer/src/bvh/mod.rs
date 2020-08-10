mod apply;
mod node;
mod strategy;
mod traverse;

pub use node::*;
use std::{cmp::Ordering, ops::Range};
pub use strategy::*;

pub trait BVHBounding: Sized + Copy {
  type AxisType: Copy;
  type CenterType;
  fn get_center(&self) -> Self::CenterType;
  fn from_groups(iter: impl Iterator<Item = Self>) -> Self;
  fn get_partition_axis(
    node: &FlattenBVHNode<Self>,
    build_source: &Vec<BuildPrimitive<Self>>,
    index_source: &Vec<usize>,
  ) -> Self::AxisType;
  fn compare(
    self_primitive: &BuildPrimitive<Self>,
    axis: Self::AxisType,
    other_primitive: &BuildPrimitive<Self>,
  ) -> Ordering;
}

pub struct BuildPrimitive<B: BVHBounding> {
  bounding: B,
  center: B::CenterType,
}

impl<B: BVHBounding> BuildPrimitive<B> {
  fn new(bounding: B) -> Self {
    Self {
      bounding,
      center: bounding.get_center(),
    }
  }

  fn compare_center(&self, axis: B::AxisType, other: &BuildPrimitive<B>) -> Ordering {
    B::compare(self, axis, &other)
  }
}

pub struct BVHOption {
  pub max_tree_depth: usize,
  pub bin_size: usize,
}

impl Default for BVHOption {
  fn default() -> Self {
    Self {
      max_tree_depth: 10,
      bin_size: 1,
    }
  }
}

pub struct FlattenBVH<B: BVHBounding, S: BVHBuildStrategy<B>> {
  nodes: Vec<FlattenBVHNode<B>>,
  sorted_primitive_index: Vec<usize>,
  option: BVHOption,
  strategy: S,
}

impl<B: BVHBounding, S: BVHBuildStrategy<B>> FlattenBVH<B, S> {
  pub fn new(source: impl ExactSizeIterator<Item = B>, strategy: S, option: BVHOption) -> Self {
    // prepare build source;
    let items_count = source.len();
    let (mut index_list, primitives) = source
      .enumerate()
      .map(|(i, b)| (i, BuildPrimitive::new(b)))
      .unzip();

    // prepare root
    let root_bbox = bounding_from_build_source(&index_list, &primitives, 0..items_count);

    let mut nodes = Vec::new();
    nodes.push(FlattenBVHNode::new(root_bbox, 0..items_count, 0, 0));

    // build
    strategy.build(&option, &primitives, &mut index_list, &mut nodes);

    Self {
      nodes,
      sorted_primitive_index: index_list,
      option,
      strategy,
    }
  }

  pub fn option(&self) -> &BVHOption {
    &self.option
  }

  pub fn strategy(&self) -> &S {
    &self.strategy
  }

  pub fn sorted_primitive_index(&self) -> &Vec<usize> {
    &self.sorted_primitive_index
  }
}

fn bounding_from_build_source<B: BVHBounding>(
  index_list: &Vec<usize>,
  primitives: &Vec<BuildPrimitive<B>>,
  range: Range<usize>,
) -> B {
  B::from_groups(
    index_list
      .get(range.clone())
      .unwrap()
      .iter()
      .map(|index| primitives[*index].bounding),
  )
}
