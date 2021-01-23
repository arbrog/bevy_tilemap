use crate::chunk::base_tile::BaseTile;
use crate::lib::*;

/// Raw tile that is stored in the chunks.
pub mod base_tile;
/// Chunk entity.
pub(crate) mod entity;
/// Sparse and dense chunk layers.
mod layer;
/// Meshes for rendering to vertices.

/// Files and helpers for rendering.
pub(crate) mod render;

pub trait Chunk<T: BaseTile> {
    fn point(&self) -> Point2;

    fn set_entity(&mut self, z_order: usize, entity: Entity);

    fn get_entity(&self, z_order: usize) -> Option<Entity>;

    fn get_entities(&self) -> Vec<Entity>;

    fn get_tile(&self, z_order: usize, index: usize) -> Option<&T>;

    fn get_tile_mut(&mut self, z_order: usize, index: usize) -> Option<&mut T>;
}

pub trait ChunkRender {
    fn set_mesh(&mut self, z_order: usize, mesh: Handle<Mesh>);

    fn tiles_to_renderer_parts(
        &self,
        z_order: usize,
        dimensions: Dimension2,
    ) -> Option<(Vec<f32>, Vec<[f32; 4]>)>;
}
