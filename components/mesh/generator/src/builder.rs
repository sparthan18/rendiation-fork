use rendiation_renderable_mesh::group::MeshGroupsInfo;

use crate::*;

pub enum DynIndex {
  Uint16(Vec<u16>),
  Uint32(Vec<u32>),
}

/// Mark type that indicates index oversized u32 and cannot used in gpu.
pub struct IndexOversized;

impl DynIndex {
  pub fn is_u32_buffer(&self) -> bool {
    match self {
      DynIndex::Uint16(_) => false,
      DynIndex::Uint32(_) => true,
    }
  }

  pub fn push_index(&mut self, index: usize) -> Result<(), IndexOversized> {
    if index > u32::MAX as usize {
      Err(IndexOversized)
    } else {
      match self {
        DynIndex::Uint16(buffer) => {
          if index > u16::MAX as usize {
            let buffer = self.try_upgrade_to_u32();
            buffer.push(index as u32)
          } else {
            buffer.push(index as u16)
          }
        }
        DynIndex::Uint32(buffer) => buffer.push(index as u32),
      }
      Ok(())
    }
  }

  pub fn try_upgrade_to_u32(&mut self) -> &mut Vec<u32> {
    match self {
      DynIndex::Uint16(buffer) => {
        *self = DynIndex::Uint32(buffer.iter().map(|&i| i as u32).collect());
        self.try_upgrade_to_u32()
      }
      DynIndex::Uint32(buffer) => buffer,
    }
  }
}

pub struct IndexedMeshBuilder<I, U, T, V> {
  index: Vec<I>,
  container: U,
  phantom1: PhantomData<T>,
  phantom2: PhantomData<V>,
  groups: MeshGroupsInfo,
}

impl<I, U, T, V> IndexedMeshBuilder<I, U, T, V> {
  pub fn build_mesh(self) -> IndexedMesh<I, V, T, U> {
    IndexedMesh::new(self.container, self.index)
  }
}

#[derive(Copy, Clone)]
pub struct TessellationConfig {
  pub u: usize,
  pub v: usize,
}

pub trait VertexContainer {
  type Vertex;
  fn push_vertex(&mut self, v: Self::Vertex);
}

pub trait VertexBuilding {
  fn from_surface(surface: &impl ParametricSurface, uv: Vec2<f32>) -> Self;
}

impl<I, U, V> IndexedMeshBuilder<I, U, TriangleList, V> {
  pub fn triangulate_parametric(
    &mut self,
    surface: &impl ParametricSurface,
    config: TessellationConfig,
    keep_grouping: bool,
  ) where
    V: VertexBuilding,
    U: VertexContainer<Vertex = V>,
    I: From<usize> + Copy,
  {
    let u_step = 1. / config.u as f32;
    let v_step = 1. / config.v as f32;
    for u in 0..config.u {
      for v in 0..config.v {
        let u = u as f32 * u_step;
        let v = v as f32 * v_step;
        let vertex = V::from_surface(surface, (u, v).into());
        self.container.push_vertex(vertex)
      }
    }

    let index_start = self.index.len();
    let uv_to_index = |u: usize, v: usize| -> I { (index_start + u + config.u * v).into() };

    for u in 0..config.u {
      for v in 0..config.v {
        // a  b
        // c  d
        let a = uv_to_index(u, v);
        let b = uv_to_index(u, v + 1);
        let c = uv_to_index(u + 1, v);
        let d = uv_to_index(u + 1, v + 1);

        self.index.push(a);
        self.index.push(c);
        self.index.push(b);

        self.index.push(b);
        self.index.push(c);
        self.index.push(d);
      }
    }

    let count = config.u * config.v * 6;
    if keep_grouping {
      self.groups.push_consequent(count);
    } else {
      self.groups.extend_last(count)
    }
  }
}

impl<I, U, V> IndexedMeshBuilder<I, U, LineList, V> {
  pub fn build_grid_parametric(
    &mut self,
    surface: &impl ParametricSurface,
    config: TessellationConfig,
    keep_grouping: bool,
  ) where
    V: VertexBuilding,
    U: VertexContainer<Vertex = V>,
    I: From<usize> + Copy,
  {
    let u_step = 1. / config.u as f32;
    let v_step = 1. / config.v as f32;
    for u in 0..config.u {
      for v in 0..config.v {
        let u = u as f32 * u_step;
        let v = v as f32 * v_step;
        let vertex = V::from_surface(surface, (u, v).into());
        self.container.push_vertex(vertex)
      }
    }

    let index_start = self.index.len();
    let uv_to_index = |u: usize, v: usize| -> I { (index_start + u + config.u * v).into() };

    for u in 0..config.u {
      for v in 0..config.v {
        // a  b
        // c  d
        let a = uv_to_index(u, v);
        let b = uv_to_index(u, v + 1);
        let c = uv_to_index(u + 1, v);
        let d = uv_to_index(u + 1, v + 1);

        self.index.push(a);
        self.index.push(b);

        self.index.push(a);
        self.index.push(c);

        if u == config.u {
          self.index.push(c);
          self.index.push(d);
        }

        if v == config.v {
          self.index.push(b);
          self.index.push(d);
        }
      }
    }
    let count = config.u * config.v * 4 + config.u * 2 + config.v * 2;
    if keep_grouping {
      self.groups.push_consequent(count);
    } else {
      self.groups.extend_last(count)
    }
  }
}
