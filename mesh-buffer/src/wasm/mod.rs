use super::geometry::*;
use rendiation_ral::GeometryResourceInstance;
use rendiation_webgl::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WASMAttributeBufferF32 {
  #[wasm_bindgen(skip)]
  pub buffer: Vec<f32>,
  #[wasm_bindgen(skip)]
  pub stride: usize,
}

#[wasm_bindgen]
pub struct WASMAttributeBufferU16 {
  #[wasm_bindgen(skip)]
  pub buffer: Vec<u16>,
  #[wasm_bindgen(skip)]
  pub stride: usize,
}

#[wasm_bindgen]
impl WASMAttributeBufferF32 {
  #[wasm_bindgen(constructor)]
  pub fn new(buffer: &[f32], stride: usize) -> Self {
    Self {
      buffer: buffer.to_owned(),
      stride,
    }
  }
}

#[wasm_bindgen]
impl WASMAttributeBufferU16 {
  #[wasm_bindgen(constructor)]
  pub fn new(buffer: &[u16], stride: usize) -> Self {
    Self {
      buffer: buffer.to_owned(),
      stride,
    }
  }
}

#[wasm_bindgen]
pub struct WASMGeometry {
  // data: GeometryResourceInstance<WebGLRenderer>,
  index: Option<usize>,
  position: usize,
  normal: Option<usize>,
  uv: Option<usize>,
}

impl WASMGeometry {
  pub fn to_geometry_resource_instance(&self) -> GeometryResourceInstance<WebGLRenderer> {
    todo!()
  }
}

#[wasm_bindgen]
impl WASMGeometry {
  #[wasm_bindgen(constructor)]
  pub fn new(
    index: Option<usize>,
    position: usize,
    normal: Option<usize>,
    uv: Option<usize>,
  ) -> Self {
    Self {
      index,
      position,
      normal,
      uv,
    }
  }
}