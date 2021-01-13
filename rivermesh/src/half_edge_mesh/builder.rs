use arena::{Arena, Handle};

use crate::{HalfEdge, HalfEdgeMesh, HalfEdgeMeshData, HalfEdgeVertex};

pub enum BuildingVertex<M: HalfEdgeMeshData> {
  Detached(M::Vertex),
  Attached(Handle<HalfEdgeVertex<M>>),
}

impl<M: HalfEdgeMeshData> BuildingVertex<M> {
  pub fn is_attached(&self) -> bool {
    match self {
      BuildingVertex::Detached(_) => false,
      BuildingVertex::Attached(_) => true,
    }
  }

  pub fn is_same_and_attached(&self, other: &Self) -> bool {
    if let BuildingVertex::Attached(handle_self) = self {
      if let BuildingVertex::Attached(handle) = other {
        return handle_self == handle;
      }
    }
    false
  }
}

pub enum HalfEdgeBuildError {
  NonManifoldOperation,
  FaceConstructionInputTooSmall,
  TriangleInputDegenerated,
}
use HalfEdgeBuildError::*;

pub struct HalfEdgeMeshBuilder<M: HalfEdgeMeshData> {
  mesh: HalfEdgeMesh<M>,
  /// for operation recovery
  not_committed_vertices: Vec<Handle<HalfEdgeVertex<M>>>,
  not_committed_half_edges: Vec<Handle<HalfEdge<M>>>,
  pub building_vertices: Arena<BuildingVertex<M>>, // this actually not allow remove, so we should not use arena!
}

impl<M: HalfEdgeMeshData> HalfEdgeMeshBuilder<M> {
  pub fn new() -> Self {
    Self {
      mesh: HalfEdgeMesh::new(),
      not_committed_vertices: Vec::new(),
      not_committed_half_edges: Vec::new(),
      building_vertices: Arena::new(),
    }
  }

  fn recovery(&mut self) {
    let mesh = &mut self.mesh;
    self.not_committed_vertices.drain(..).for_each(|h| {
      mesh.vertices.remove(h);
    });
    self.not_committed_half_edges.drain(..).for_each(|h| {
      mesh.half_edges.remove(h);
    });
  }

  pub fn push_any_face(&mut self, path: &[BuildingVertex<M>]) -> Result<(), HalfEdgeBuildError> {
    if path.len() < 3 {
      return Err(FaceConstructionInputTooSmall);
    }
    todo!()
  }

  pub fn push_triangle_face(
    &mut self,
    a: BuildingVertex<M>,
    b: BuildingVertex<M>,
    c: BuildingVertex<M>,
  ) -> Result<(), HalfEdgeBuildError> {
    let result = self.push_triangle_face_impl(a, b, c);
    if result.is_err() {
      self.recovery()
    }
    result
  }

  fn insert_building_vertex(
    &mut self,
    v: BuildingVertex<M>,
  ) -> Result<(Handle<HalfEdgeVertex<M>>, bool), HalfEdgeBuildError> {
    match v {
      BuildingVertex::Detached(v) => {
        let vertex = HalfEdgeVertex {
          data: v,
          edge: Handle::from_raw_parts(0, 0),
        };
        let inserted = self.mesh.vertices.insert(vertex);
        self.not_committed_vertices.push(inserted);
        Ok((inserted, true))
      }
      BuildingVertex::Attached(v) => {
        let vertex = self.mesh.vertices.get(v).unwrap();
        if vertex.is_boundary_vertex(&self.mesh) {
          Err(HalfEdgeBuildError::NonManifoldOperation) // dangling point/edge
        } else {
          Ok((v, false))
        }
      }
    }
  }

  fn insert_building_half_edge(
    &mut self,
    from: (Handle<HalfEdgeVertex<M>>, bool),
    to: (Handle<HalfEdgeVertex<M>>, bool),
  ) -> Result<Handle<HalfEdge<M>>, HalfEdgeBuildError> {
    if !from.1 && !from.1 {
      if HalfEdge::check_exist(&self.mesh, from.0, to.0) {
        return Err(HalfEdgeBuildError::NonManifoldOperation); // face inverted
      }
    }

    let edge = HalfEdge {
      data: M::HalfEdge::default(),
      vert: from.0,
      pair: None,
      face: Handle::from_raw_parts(0, 0),
      next: Handle::from_raw_parts(0, 0),
    };
    let inserted = self.mesh.half_edges.insert(edge);
    self.not_committed_half_edges.push(inserted);
    Ok(inserted)
  }

  pub fn push_triangle_face_impl(
    &mut self,
    a: BuildingVertex<M>,
    b: BuildingVertex<M>,
    c: BuildingVertex<M>,
  ) -> Result<(), HalfEdgeBuildError> {
    if a.is_same_and_attached(&b) || b.is_same_and_attached(&c) || c.is_same_and_attached(&a) {
      return Err(TriangleInputDegenerated);
    }

    let a = self.insert_building_vertex(a)?;
    let b = self.insert_building_vertex(b)?;
    let c = self.insert_building_vertex(c)?;
    let ab = self.insert_building_half_edge(a, b)?;
    let bc = self.insert_building_half_edge(b, c)?;
    let ca = self.insert_building_half_edge(c, a)?;

    Ok(())
  }

  pub fn done(self) -> HalfEdgeMesh<M> {
    self.mesh
  }
}
