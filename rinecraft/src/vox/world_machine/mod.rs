use crate::vox::block::Block;
use crate::vox::block_meta::BlockRegistry;
use rendiation::*;

pub trait WorldMachine {
  fn world_gen(&self, x: i32, y: i32, z: i32) -> Block;
}

pub struct WorldMachineImpl {
  pub block_registry: BlockRegistry,
  version: usize,
  seed: usize,
}

impl WorldMachineImpl {
  pub fn new() -> Self {
    let block_registry = BlockRegistry::new_default();
    WorldMachineImpl {
      block_registry,
      version: 0,
      seed: 0,
    }
  }

  pub fn create_block_atlas_gpu(&mut self, renderer: &mut WGPURenderer) -> wgpu::TextureView{
    let texture = self.block_registry.create_atlas(renderer);
    texture.make_default_view()
  }
}

pub const STONE: Block = Block::new(0);
impl WorldMachine for WorldMachineImpl {
  fn world_gen(&self, x: i32, y: i32, z: i32) -> Block {
    if y <= x.abs() && y <= z.abs() {
      STONE
    } else {
      Block::void()
    }
  }
}