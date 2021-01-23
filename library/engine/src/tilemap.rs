use crate::chunk::base_tile::BaseTile;
use crate::chunk::render::GridTopology;
use crate::chunk::Chunk;
use crate::event::{TilemapChunkDespawned, TilemapChunkModified};
use crate::{event::TilemapChunkSpawned, lib::*};

pub struct TileLayer {
    pub kind: LayerKind,
}

pub trait Tilemap<T: BaseTile, C: Chunk<T>> {
    // type Dimension: Copy + Clone + Eq + PartialEq + Ord + PartialOrd + Hash + Debug;
    // type Point;
    //
    // fn set_texture_atlas(&self) -> &Handle<TextureAtlas>;
    //
    // fn set_tile_origin(&self) -> TileOrigin;

    //
    fn texture_atlas(&self) -> &Handle<TextureAtlas>;

    //
    fn texture_dimensions(&self) -> Dimension2;

    //
    fn tile_dimensions(&self) -> DimensionKind;

    // fn insert_tiles<P, I>(&mut self, tiles: I) -> Result<(), Self::Error>
    // where
    //     P: Into<Self::Point>,
    //     I: IntoIterator<Item = Tile<P>>;
    //
    // fn insert_tile<P: Into<Self::Point>>(&mut self, tile: Tile<P>) -> Result<(), Self::Error>;
    //
    // fn clear_tiles<P, I>(&mut self, points: I) -> Result<(), Self::Error>
    // where
    //     P: Into<Self::Point>,
    //     I: IntoIterator<Item = (P, usize)>;
    //
    // fn clear_tile<P>(&mut self, point: P, z_order: usize) -> Result<(), Self::Error>;
    //
    // fn clear(&mut self, z_order: usize) -> Result<(), Self::Error>;
    //
    // fn get_tile<P>(&mut self, point: P, z_order: usize) -> Option<&Rawtile>
    // where
    //     P: Into<Self::Point>;
    //
    // fn get_tile_mut<P>(&mut self, point: P, z_order: usize) -> Option<&mut RawTile>
    // where
    //     P: Into<Self::Point>;
    //
    // fn set_layer(&mut self, layer: TileLayer) -> Result<(), Self::Error>;
    //
    // fn move_layer(&mut self, from_z_order: usize, to_z_order: usize) -> Result<(), Self::Error>;
    //
    // fn remove_layer(&mut self, z_order: usize) -> Result<(), Self::Error>;

    //
    fn layers(&self) -> Vec<Option<TileLayer>>;

    //
    fn topology(&self) -> GridTopology;

    //
    fn chunk_dimensions(&self) -> DimensionKind;

    // fn insert_chunk<T, P: Into<Point2>>(&mut self, point: P) -> Result<(), Self::Error>;
    //
    // fn contains_chunk<T, P: Into<Point2>>(&mut self, point: P) -> bool;
    //
    // fn set_auto_spawn(&mut self, dimension: Dimension2);
    //
    // fn spawn_chunk<P: Into<Point2>>(&mut self, point: P) -> Result<(), Self::Error>;
    //
    // fn spawn_chunk_containing_point<P: Into<Point2>>(
    //     &mut self,
    //     point: P,
    // ) -> Result<(), Self::Error>;
    //
    // fn despawn_chunk<P: Into<Point2>>(&mut self, point: P) -> Result<(), Self::Error>;
    //
    // fn remove_chunk<P: Into<Point2>>(&mut self, point: P) -> Result<(), Self::Error>;

    //
    fn chunks(&self) -> &HashMap<Point2, C>;

    //
    fn chunks_mut(&mut self) -> &mut HashMap<Point2, C>;

    fn point_to_chunk_point<P: Into<Point2>>(&self, point: P) -> (i32, i32);
}

pub trait TilemapEvents {
    fn chunk_spawned_events(&self) -> &Events<TilemapChunkSpawned>;

    fn chunk_modified_events(&self) -> &Events<TilemapChunkModified>;

    fn chunk_despawned_events(&self) -> &Events<TilemapChunkDespawned>;

    fn update_events(&mut self);
}
