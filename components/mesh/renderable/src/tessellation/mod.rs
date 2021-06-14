use crate::{
  group::MeshGroupsInfo,
  mesh::{AbstractMesh, IndexType, IndexedMesh, NoneIndexedMesh, TriangleList},
  vertex::Vertex,
};

pub mod cube;
pub mod cylinder;
pub mod plane;
pub mod sphere;
pub use cube::*;
pub use cylinder::*;
pub use plane::*;
pub use sphere::*;

// todo add support for index overflow check
pub trait IndexedMeshTessellator<T = Vertex, I: IndexType = u16, P = TriangleList> {
  fn tessellate(&self) -> TesselationResult<IndexedMesh<I, T, P>>;
}

pub trait NoneIndexedMeshTessellator<T = Vertex, P = TriangleList> {
  fn tessellate(&self) -> TesselationResult<NoneIndexedMesh<T, P>>;
}

pub struct TesselationResult<T> {
  pub mesh: T,
  pub range: MeshGroupsInfo,
}

impl<T: AbstractMesh> TesselationResult<T> {
  pub fn new(mesh: T, range: MeshGroupsInfo) -> Self {
    Self { mesh, range }
  }
  pub fn full_range(mesh: T) -> Self {
    let range = MeshGroupsInfo::full_range(&mesh);
    Self { mesh, range }
  }
}
