use crate::ImmediateRenderableContent;
use rendiation_mesh_buffer::{geometry::*, tessellation::*, vertex::*};
use rendiation_ral::*;

pub struct FullScreenQuad<T: RAL, SP: ShadingProvider<T, Geometry = Vertex>> {
  obj: Drawcall<T, Vertex, SP>,
}

pub struct FullScreenQuadFactory<T: RAL> {
  geometry: GeometryHandle<T, Vertex>,
}

impl<T: RAL> FullScreenQuadFactory<T> {
  pub fn new(res: &mut ResourceManager<T>, renderer: &mut T::Renderer) -> Self {
    let geometry = Quad.create_mesh(&());
    let geometry = IndexedGeometry::<_, TriangleList>::new(geometry.0, geometry.1);
    let geometry = geometry.create(res, renderer);
    let geometry = res.add_geometry(geometry);
    Self { geometry }
  }

  pub fn create_quad<SP: ShadingProvider<T, Geometry = Vertex>>(
    &self,
    shading: ShadingHandle<T, SP>,
  ) -> FullScreenQuad<T, SP> {
    FullScreenQuad {
      obj: Drawcall {
        geometry: self.geometry,
        shading,
      },
    }
  }
}

impl<T: RAL, SP: ShadingProvider<T, Geometry = Vertex>> ImmediateRenderableContent<T>
  for FullScreenQuad<T, SP>
{
  fn render(&self, pass: &mut T::RenderPass, res: &ResourceManager<T>) {
    T::render_drawcall(&self.obj, pass, res)
  }

  fn prepare(&mut self, renderer: &mut T::Renderer, resource: &mut ResourceManager<T>) {
    resource.maintain_gpu(renderer)
  }
}