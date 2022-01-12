use image::*;
use rendiation_algebra::*;
use rendiation_renderable_mesh::tessellation::{
  CubeMeshParameter, IndexedMeshTessellator, SphereMeshParameter,
};
use rendiation_texture::{rgb_to_rgba, TextureSampler, WrapAsTexture2DSource};
use rendiation_webgpu::WebGPUTexture2dSource;

use crate::*;

pub fn load_img(path: &str) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
  use image::io::Reader as ImageReader;
  let img = ImageReader::open(path).unwrap().decode().unwrap();
  match img {
    image::DynamicImage::ImageRgba8(img) => img,
    image::DynamicImage::ImageRgb8(img) => rgb_to_rgba(img),
    _ => panic!("unsupported texture type"),
  }
}

pub fn load_img_cube() -> TextureCubeSource {
  use std::convert::TryInto;
  let path = vec![
    "C:/Users/mk/Desktop/rrf-resource/Park2/posx.jpg",
    "C:/Users/mk/Desktop/rrf-resource/Park2/negx.jpg",
    "C:/Users/mk/Desktop/rrf-resource/Park2/posy.jpg",
    "C:/Users/mk/Desktop/rrf-resource/Park2/negy.jpg",
    "C:/Users/mk/Desktop/rrf-resource/Park2/posz.jpg",
    "C:/Users/mk/Desktop/rrf-resource/Park2/negz.jpg",
  ];

  fn load(path: &&str) -> Box<dyn WebGPUTexture2dSource> {
    Box::new(load_img(path).into_source())
  }

  // todo this is awkward
  let res: Vec<Box<dyn WebGPUTexture2dSource>> = path.iter().map(load).collect();

  unsafe { res.try_into().unwrap_unchecked() }
}

pub fn load_default_scene(scene: &mut Scene) {
  let path = if cfg!(windows) {
    "C:/Users/mk/Desktop/rrf-resource/planets/earth_atmos_2048.jpg"
  } else {
    "/Users/mikialex/Desktop/test.png"
  };

  let texture = SceneTexture2D::new(Box::new(load_img(path).into_source()));

  // let texture_cube = scene.add_texture_cube(load_img_cube());

  // let background_mat = EnvMapBackGroundMaterial {
  //   sampler: TextureSampler::default(),
  //   texture: texture_cube,
  // };
  // let background_mat = scene.add_material(background_mat);
  // let bg = DrawableBackground::new(background_mat);

  // scene.background = Box::new(bg);

  {
    let mesh = SphereMeshParameter::default().tessellate();
    let mesh = MeshCell::new(MeshSource::new(mesh));
    let material = PhysicalMaterial {
      albedo: Vec3::splat(1.),
      sampler: TextureSampler::default(),
      texture: texture.clone(),
    }
    .into_scene_material()
    .into_resourced();

    let child = scene.root.create_child();
    child.mutate(|node| node.local_matrix = Mat4::translate(2., 0., 3.));

    let model = MeshModel::new(material, mesh, child);
    scene.add_model(model)
  }

  {
    let mesh = CubeMeshParameter::default().tessellate();
    let mesh = MeshCell::new(MeshSource::new(mesh));
    let mut material = PhysicalMaterial {
      albedo: Vec3::splat(1.),
      sampler: TextureSampler::default(),
      texture,
    }
    .into_scene_material()
    .into_resourced();
    material.states.depth_compare = wgpu::CompareFunction::Always;

    let model = MeshModel::new(material, mesh, scene.root.create_child());
    scene.add_model(model)
  }

  {
    let camera = PerspectiveProjection::default();
    let camera_node = scene.root.create_child();
    camera_node.mutate(|node| {
      node.local_matrix = Mat4::lookat(Vec3::splat(3.), Vec3::splat(0.), Vec3::new(0., 1., 0.));
    });
    let camera = SceneCamera::new(camera, camera_node);

    scene.active_camera = camera.into();
  }

  {
    let camera = PerspectiveProjection::default();
    let camera_node = scene.root.create_child();
    camera_node.mutate(|node| {
      node.local_matrix = Mat4::lookat(Vec3::splat(3.), Vec3::splat(0.), Vec3::new(0., 1., 0.));
    });
    let camera = SceneCamera::new(camera, camera_node);

    scene.cameras.insert(camera);
  }
}
