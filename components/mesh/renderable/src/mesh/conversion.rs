//! The conversion method between different mesh types

// todo for convert between different mesh type

// downgrade:
// mesh -> line , wireframe? edge?
// line -> point

// indexed -> noneIndexed expand?
// noneIndexed -> indexed indexed?

use std::hash::Hash;
use std::{cmp::Ordering, iter::FromIterator, ops::Deref};

use fast_hash_collection::*;
use rendiation_algebra::{InnerProductSpace, Vec3};
use rendiation_geometry::{LineSegment, Triangle};

use crate::*;

impl<T, U, IU> IndexedMesh<T, U, IU>
where
  for<'a> IndexView<'a, Self>: AbstractMesh<Primitive = Triangle<IU::Output>>,
  Self: AbstractMesh,
  IU: IndexContainer,
  IU::Output: IndexType,
  U: Clone,
{
  pub fn create_wireframe<RIU>(&self) -> IndexedMesh<LineList, U, RIU>
  where
    RIU: FromIterator<IU::Output>,
  {
    let mut deduplicate_set = FastHashSet::<LineSegment<IU::Output>>::default();
    self
      .primitive_iter()
      .zip(self.as_index_view().primitive_iter())
      .for_each(|(_, f)| {
        f.for_each_edge(|edge| {
          deduplicate_set.insert(edge.swap_if(|l| l.start < l.end));
        })
      });
    let new_index = deduplicate_set
      .iter()
      .flat_map(|l| l.iter_point().copied())
      .collect();
    IndexedMesh::new(self.vertex.clone(), new_index)
  }
}

impl<T, U, IU> IndexedMesh<T, U, IU>
where
  for<'a> IndexView<'a, Self>: AbstractMesh<Primitive = Triangle<IU::Output>>,
  Self: AbstractMesh<Primitive = Triangle<U::Output>>,
  U: VertexContainer + FromIterator<U::Output>,
  IU: IndexContainer,
  IU::Output: IndexType,
  U::Output: Copy,
  U::Output: Deref<Target = Vec3<f32>>,
{
  /// maybe you should merge vertex before create edge
  /// non manifold mesh may affect result
  pub fn create_edge(&self, edge_threshold_angle: f32) -> NoneIndexedMesh<LineList, U> {
    // Map: edge id => (edge face idA, edge face idB(optional));
    let mut edges = FastHashMap::<LineSegment<IU::Output>, (usize, Option<usize>)>::default();
    self
      .primitive_iter()
      .zip(self.as_index_view().primitive_iter())
      .enumerate()
      .for_each(|(face_id, (_, f))| {
        f.for_each_edge(|edge| {
          edges
            .entry(edge.swap_if(|l| l.start < l.end))
            .and_modify(|e| e.1 = Some(face_id))
            .or_insert_with(|| (face_id, None));
        })
      });
    let normals = self
      .primitive_iter()
      .map(|f| f.map(|v| *v).face_normal().value)
      .collect::<Vec<Vec3<f32>>>();
    let threshold_dot = edge_threshold_angle.cos();
    let data = edges
      .iter()
      .filter(|(_, f)| f.1.is_none() || normals[f.0].dot(normals[f.1.unwrap()]) <= threshold_dot)
      .map(|(e, _)| e)
      .flat_map(|l| l.iter_point())
      .map(|i| self.vertex.index_get(i.into_usize()).unwrap())
      .collect();
    NoneIndexedMesh::new(data)
  }
}

impl<T, U, IU> IndexedMesh<T, U, IU>
where
  IU: IndexContainer + TryFromIterator<usize>,
  for<'a> &'a IU: IntoIterator<Item = IU::Output>,
  for<'a> &'a U: IntoIterator<Item = &'a U::Output>,
  U: VertexContainer,
  IU::Output: IndexType,
  U::Output: Copy,
{
  pub fn merge_vertex_by_sorting(
    &self,
    mut sorter: impl FnMut(&U::Output, &U::Output) -> Ordering,
    mut merger: impl FnMut(&U::Output, &U::Output) -> bool,
  ) -> Result<IndexedMesh<T, Vec<U::Output>, IU>, IU::Error> {
    let data = &self.vertex;
    let mut resorted: Vec<_> = data.into_iter().enumerate().map(|(i, v)| (i, v)).collect();
    let mut merge_data = Vec::with_capacity(resorted.len());
    let mut deduplicate_map = Vec::with_capacity(self.index.len());
    resorted.sort_unstable_by(|a, b| sorter(a.1, b.1));

    let mut resort_map: Vec<_> = (0..self.vertex.len()).collect();
    resorted
      .iter()
      .enumerate()
      .for_each(|(i, v)| resort_map[v.0] = i);

    if self.vertex.len() >= 2 {
      merge_data.push(*resorted[0].1);
      deduplicate_map.push(0);

      resorted.windows(2).for_each(|v| {
        if !merger(v[0].1, v[1].1) {
          merge_data.push(*v[1].1);
        }
        deduplicate_map.push(merge_data.len() - 1);
      });
    }

    let index = &self.index;
    let new_index = IU::try_from_iter(index.into_iter().map(|i| resort_map[i.into_usize()]))?;

    Ok(IndexedMesh::new(merge_data, new_index))
  }
}

impl<T, U, IU> IndexedMesh<T, U, IU>
where
  IU: IndexContainer,
  IU::Output: IndexType,
  U: IndexGet + FromIterator<U::Output>,
  for<'a> &'a IU: IntoIterator<Item = IU::Output>,
{
  pub fn expand_to_none_index_geometry(&self) -> NoneIndexedMesh<T, U> {
    let index = &self.index;
    NoneIndexedMesh::new(
      index
        .into_iter()
        .map(|i| self.vertex.index_get((i).into_usize()).unwrap())
        .collect(),
    )
  }
}

impl<T, U> NoneIndexedMesh<T, U>
where
  U: VertexContainer,
  U::Output: Eq + Hash + Copy,
  for<'a> &'a U: IntoIterator<Item = &'a U::Output>,
  Self: AbstractMesh,
{
  pub fn create_index_geometry<IU>(&self) -> Result<IndexedMesh<T, Vec<U::Output>, IU>, IU::Error>
  where
    IU: TryFromIterator<usize>,
  {
    let mut deduplicate_map = FastHashMap::<U::Output, usize>::default();
    let mut deduplicate_buffer = Vec::with_capacity(self.data.len());
    let data = &self.data;
    let index = IU::try_from_iter(data.into_iter().map(|v| {
      *deduplicate_map.entry(*v).or_insert_with(|| {
        deduplicate_buffer.push(*v);
        deduplicate_buffer.len() - 1
      })
    }))?;
    deduplicate_buffer.shrink_to_fit();
    Ok(IndexedMesh::new(deduplicate_buffer, index))
  }
}

impl<T, U, IU> IndexedMesh<T, U, IU>
where
  U: Clone,
{
  pub fn create_point_cloud(&self) -> NoneIndexedMesh<PointList, U> {
    NoneIndexedMesh::new(self.vertex.clone())
  }
}
