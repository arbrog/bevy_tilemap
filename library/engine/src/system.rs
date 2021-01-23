use crate::chunk::base_tile::BaseTile;
use crate::chunk::{Chunk, ChunkRender};
use crate::{
    chunk::{
        entity::{ChunkBundle, ModifiedLayer, ZOrder},
        render::{mesh::ChunkMesh, GridTopology},
    },
    lib::*,
    tilemap::{Tilemap, TilemapEvents},
    SpawnedChunks,
};

pub(crate) fn tilemap_update_events<T: TilemapEvents + WorldQuery + Component>(
    mut tilemap_query: Query<&mut T>,
) {
    for mut tilemap in tilemap_query.iter_mut() {
        tilemap.update_events();
    }
}

pub(crate) fn tilemap_chunk_spawned<
    T: BaseTile,
    C: Chunk<T> + ChunkRender,
    M: Tilemap<T, C> + TilemapEvents + WorldQuery + Component,
>(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut tilemap_query: Query<(Entity, &mut M)>,
    mut spawned_chunks: ResMut<SpawnedChunks>,
) {
    for (map_entity, mut tilemap) in tilemap_query.iter_mut() {
        let mut reader = tilemap.chunk_spawned_events().get_reader();
        let reader_len = reader.iter(&tilemap.chunk_spawned_events()).count();
        for event in reader.iter(&tilemap.chunk_spawned_events()) {
            let point: Point2 = event.point;
            let spawned_map = spawned_chunks
                .0
                .entry(map_entity)
                .or_insert_with(|| HashSet::default());
            if spawned_map.contains(&point) {
                continue;
            } else {
                spawned_map.insert(point);
            }

            let layers = tilemap.layers();
            let layers_len = tilemap.layers().len();
            let chunk_dimensions = tilemap.chunk_dimensions();
            let tile_dimensions = tilemap.tile_dimensions();
            let texture_dimensions = tilemap.texture_dimensions();
            let texture_atlas = tilemap.texture_atlas().clone_weak();
            let pipeline_handle = tilemap.topology().to_pipeline_handle();
            let topology = tilemap.topology();
            let chunk = if let Some(chunk) = tilemap.chunks_mut().get_mut(&point) {
                chunk
            } else {
                warn!("Can not get chunk at {}, skipping", &point);
                continue;
            };
            let mut entities = Vec::with_capacity(reader_len);
            for z_order in 0..layers_len {
                if layers.get(z_order).is_none() {
                    continue;
                }
                let mut mesh = Mesh::from(&ChunkMesh::new(chunk_dimensions.into()));
                let (indexes, colors) = if let Some(parts) =
                    chunk.tiles_to_renderer_parts(z_order, chunk_dimensions.into())
                {
                    parts
                } else {
                    warn!("Can not split tiles to data for the renderer");
                    continue;
                };
                mesh.set_attribute(ChunkMesh::ATTRIBUTE_TILE_INDEX, indexes);
                mesh.set_attribute(ChunkMesh::ATTRIBUTE_TILE_COLOR, colors);
                let mesh_handle = meshes.add(mesh);
                chunk.set_mesh(z_order, mesh_handle.clone());

                use DimensionKind::*;
                let (tile_width, tile_height) = match tile_dimensions {
                    Dimension2(d2) => (
                        d2.width * texture_dimensions.width,
                        d2.height * texture_dimensions.height,
                    ),
                    Dimension3(d3) => (
                        d3.width * texture_dimensions.width,
                        d3.height * texture_dimensions.height,
                    ),
                };
                let (chunk_width, chunk_height) = match chunk_dimensions {
                    Dimension2(d2) => (
                        d2.width * texture_dimensions.width,
                        d2.height * texture_dimensions.height,
                    ),
                    Dimension3(d3) => (
                        d3.width * texture_dimensions.width,
                        d3.height * texture_dimensions.height,
                    ),
                };
                use GridTopology::*;
                let translation_x = match topology {
                    HexX | HexEvenCols | HexOddCols => {
                        (((chunk.point().x * tile_width as i32) as f32 * 0.75) as i32
                            * chunk_width as i32) as f32
                    }
                    HexY => {
                        (chunk.point().x * tile_width as i32 * chunk_width as i32) as f32
                            + (chunk.point().y as f32 * chunk_height as f32 * 0.5)
                                * tile_width as f32
                    }
                    Square | HexEvenRows | HexOddRows => {
                        (chunk.point().x * tile_width as i32 * chunk_width as i32) as f32
                    }
                };
                let translation_y = match topology {
                    HexX => {
                        (chunk.point().y * tile_height as i32 * chunk_height as i32) as f32
                            + (chunk.point().x as f32 * chunk_width as f32 * 0.5)
                                * tile_height as f32
                    }
                    HexY | HexEvenRows | HexOddRows => {
                        (((chunk.point().y * tile_height as i32) as f32 * 0.75) as i32
                            * chunk_height as i32) as f32
                    }
                    Square | HexEvenCols | HexOddCols => {
                        (chunk.point().y * tile_height as i32 * chunk_height as i32) as f32
                    }
                };
                let translation = Vec3::new(translation_x, translation_y, z_order as f32);
                let pipeline = RenderPipeline::new(pipeline_handle.clone_weak().typed());
                let entity = if let Some(entity) = commands
                    .spawn(ChunkBundle {
                        point,
                        z_order: ZOrder(z_order),
                        texture_atlas: texture_atlas.clone_weak(),
                        mesh: mesh_handle.clone_weak(),
                        transform: Transform::from_translation(translation),
                        render_pipelines: RenderPipelines::from_pipelines(vec![pipeline]),
                        draw: Default::default(),
                        visible: Visible {
                            // TODO: this would be nice as a config parameter to make
                            // RapierRenderPlugin's output visible.
                            is_visible: true,
                            is_transparent: true,
                        },
                        main_pass: MainPass,
                        global_transform: Default::default(),
                        modified_layer: Default::default(),
                    })
                    .current_entity()
                {
                    entity
                } else {
                    error!(
                        "Chunk entity does not exist unexpectedly, can not run the tilemap system"
                    );
                    return;
                };

                info!("Chunk {} spawned", point);

                chunk.set_entity(z_order, entity);
                entities.push(entity);
            }
            commands.push_children(map_entity, &entities);
        }
    }
}

pub(crate) fn tilemap_chunk_modified<
    T: BaseTile,
    C: Chunk<T>,
    M: Tilemap<T, C> + TilemapEvents + WorldQuery + Component,
>(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut tilemap_query: Query<(Entity, &mut M)>,
    mut layer_query: Query<&mut ModifiedLayer>,
) {
    for (map_entity, mut tilemap) in tilemap_query.iter_mut() {
        let mut reader = tilemap.chunk_modified_events().get_reader();
        let reader_len = reader.iter(&tilemap.chunk_modified_events()).count();
        for event in reader.iter(&tilemap.chunk_modified_events()) {
            let layer = event.layer;
            for entity in layer.into_iter() {
                let mut modified_layer = if let Ok(layer) = layer_query.get_mut(entity) {
                    layer
                } else {
                    warn!("Chunk layer does not exist, skipping");
                    continue;
                };
                modified_layer.0 += 1;
            }
        }
    }
}

/// The event handling system for the tilemap.
///
/// There are a few things that happen in this function which are outlined in
/// order of operation here. It was done in this order that made the most sense
/// at the time of creation.
///
/// 1. Spawn chunks
/// 1. Modify chunks
/// 1. Despawn chunks
pub(crate) fn tilemap_events<
    T: BaseTile,
    C: Chunk<T>,
    M: Tilemap<T, C> + WorldQuery + Component,
>(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut tilemap_query: Query<(Entity, &mut M)>,
    mut layer_query: Query<&mut ModifiedLayer>,
) {
    for (map_entity, mut tilemap) in tilemap_query.iter_mut() {
        let mut modified_chunks = Vec::new();
        let mut spawned_chunks = Vec::new();
        let mut despawned_chunks = Vec::new();
        let mut reader = tilemap.chunk_events().get_reader();
        for event in reader.iter(&tilemap.chunk_events()) {
            use TilemapChunkEvent::*;
            match event {
                Modified { ref layers } => {
                    modified_chunks.push(layers.clone());
                }
                Spawned { ref point } => {
                    spawned_chunks.push(*point);
                }
                Despawned {
                    ref entities,
                    ref point,
                } => {
                    despawned_chunks.push((entities.clone(), *point));
                }
            }
        }

        let capacity = spawned_chunks.len();
        for point in spawned_chunks.into_iter() {
            if tilemap.spawned_chunks().contains(&(point.x, point.y)) {
                continue;
            } else {
                tilemap.spawned_chunks_mut().insert((point.x, point.y));
            }

            let layers = tilemap.layers();
            let layers_len = tilemap.layers().len();
            let chunk_dimensions = tilemap.chunk_dimensions();
            let tile_dimensions = tilemap.tile_dimensions();
            let texture_atlas = tilemap.texture_atlas().clone_weak();
            let pipeline_handle = tilemap.topology().to_pipeline_handle();
            let topology = tilemap.topology();
            let chunk = if let Some(chunk) = tilemap.chunks_mut().get_mut(&point) {
                chunk
            } else {
                warn!("Can not get chunk at {}, skipping", &point);
                continue;
            };
            let mut entities = Vec::with_capacity(capacity);
            for z_order in 0..layers_len {
                if layers.get(z_order).is_none() {
                    continue;
                }
                let mut mesh = Mesh::from(&ChunkMesh::new(chunk_dimensions.into()));
                let (indexes, colors) =
                    if let Some(parts) = chunk.tiles_to_renderer_parts(z_order, chunk_dimensions) {
                        parts
                    } else {
                        warn!("Can not split tiles to data for the renderer");
                        continue;
                    };
                mesh.set_attribute(ChunkMesh::ATTRIBUTE_TILE_INDEX, indexes);
                mesh.set_attribute(ChunkMesh::ATTRIBUTE_TILE_COLOR, colors);
                let mesh_handle = meshes.add(mesh);
                chunk.set_mesh(z_order, mesh_handle.clone());

                use GridTopology::*;
                let translation_x = match topology {
                    HexX | HexEvenCols | HexOddCols => {
                        (((chunk.point().x * tile_dimensions.width as i32) as f32 * 0.75) as i32
                            * chunk_dimensions.width as i32) as f32
                    }
                    HexY => {
                        (chunk.point().x
                            * tile_dimensions.width as i32
                            * chunk_dimensions.width as i32) as f32
                            + (chunk.point().y as f32 * chunk_dimensions.height as f32 * 0.5)
                                * tile_dimensions.width as f32
                    }
                    Square | HexEvenRows | HexOddRows => {
                        (chunk.point().x
                            * tile_dimensions.width as i32
                            * chunk_dimensions.width as i32) as f32
                    }
                };
                let translation_y = match topology {
                    HexX => {
                        (chunk.point().y
                            * tile_dimensions.height as i32
                            * chunk_dimensions.height as i32) as f32
                            + (chunk.point().x as f32 * chunk_dimensions.width as f32 * 0.5)
                                * tile_dimensions.height as f32
                    }
                    HexY | HexEvenRows | HexOddRows => {
                        (((chunk.point().y * tile_dimensions.height as i32) as f32 * 0.75) as i32
                            * chunk_dimensions.height as i32) as f32
                    }
                    Square | HexEvenCols | HexOddCols => {
                        (chunk.point().y
                            * tile_dimensions.height as i32
                            * chunk_dimensions.height as i32) as f32
                    }
                };
                let translation = Vec3::new(translation_x, translation_y, z_order as f32);
                let pipeline = RenderPipeline::new(pipeline_handle.clone_weak().typed());
                let entity = if let Some(entity) = commands
                    .spawn(ChunkBundle {
                        point,
                        z_order: ZOrder(z_order),
                        texture_atlas: texture_atlas.clone_weak(),
                        mesh: mesh_handle.clone_weak(),
                        transform: Transform::from_translation(translation),
                        render_pipelines: RenderPipelines::from_pipelines(vec![pipeline]),
                        draw: Default::default(),
                        visible: Visible {
                            // TODO: this would be nice as a config parameter to make
                            // RapierRenderPlugin's output visible.
                            is_visible: true,
                            is_transparent: true,
                        },
                        main_pass: MainPass,
                        global_transform: Default::default(),
                        modified_layer: Default::default(),
                    })
                    .current_entity()
                {
                    entity
                } else {
                    error!(
                        "Chunk entity does not exist unexpectedly, can not run the tilemap system"
                    );
                    return;
                };

                info!("Chunk {} spawned", point);

                chunk.set_entity(z_order, entity);
                entities.push(entity);
            }
            commands.push_children(map_entity, &entities);
        }

        for layers in modified_chunks.into_iter() {
            for (_layer, entity) in layers.into_iter() {
                let mut modified_layer = if let Ok(layer) = layer_query.get_mut(entity) {
                    layer
                } else {
                    warn!("Chunk layer does not exist, skipping");
                    continue;
                };
                modified_layer.0 += 1;
            }
        }

        for (entities, point) in despawned_chunks.into_iter() {
            for entity in entities.into_iter() {
                commands.despawn_recursive(entity);
            }
            info!("Chunk {} despawned", point);
        }
    }
}

/// The chunk update system that is used to set attributes of the tiles and
/// tints if they need updating.
pub(crate) fn tilemap_chunk_update<
    T: BaseTile,
    C: Chunk<T>,
    M: Tilemap<T, C> + WorldQuery + Component,
>(
    mut meshes: ResMut<Assets<Mesh>>,
    map_query: Query<&M>,
    mut chunk_query: Query<(&Parent, &Point2, &ZOrder, &Handle<Mesh>), Changed<ModifiedLayer>>,
) {
    for (parent, point, z_order, mesh_handle) in chunk_query.iter_mut() {
        let tilemap = if let Ok(tilemap) = map_query.get(**parent) {
            tilemap
        } else {
            error!("`Tilemap` is missing, can not update chunk");
            return;
        };
        let chunk = if let Some(chunk) = tilemap.get_chunk(point) {
            chunk
        } else {
            error!("`Chunk` is missing, can not update chunk");
            return;
        };
        let mesh = if let Some(mesh) = meshes.get_mut(mesh_handle) {
            mesh
        } else {
            error!("`Mesh` is missing, can not update chunk");
            return;
        };
        let (indexes, colors) = if let Some((index, colors)) =
            chunk.tiles_to_renderer_parts(z_order.0, tilemap.chunk_dimensions())
        {
            (index, colors)
        } else {
            error!("Tiles are missing, can not update chunk");
            return;
        };
        mesh.set_attribute(ChunkMesh::ATTRIBUTE_TILE_INDEX, indexes);
        mesh.set_attribute(ChunkMesh::ATTRIBUTE_TILE_COLOR, colors);
    }
}

/// Actual method used to spawn chunks.
fn auto_spawn<T: BaseTile, C: Chunk<T>, M: Tilemap<T, C> + WorldQuery + Component>(
    camera_transform: &Transform,
    tilemap_transform: &Transform,
    tilemap: &mut M,
    spawn_dimensions: Dimension2,
) {
    let translation = camera_transform.translation - tilemap_transform.translation;
    let point_x = translation.x / tilemap.tile_width() as f32;
    let point_y = translation.y / tilemap.tile_height() as f32;
    let (chunk_x, chunk_y) = tilemap.point_to_chunk_point((point_x as i32, point_y as i32));
    let mut new_spawned: Vec<Point2> = Vec::new();
    let spawn_width = spawn_dimensions.width as i32;
    let spawn_height = spawn_dimensions.height as i32;
    for y in -spawn_width as i32..spawn_width + 1 {
        for x in -spawn_height..spawn_height + 1 {
            let chunk_x = x + chunk_x;
            let chunk_y = y + chunk_y;
            if let Some(width) = tilemap.width() {
                let width = (width / tilemap.chunk_width()) as i32 / 2;
                if chunk_x < -width || chunk_x > width {
                    continue;
                }
            }
            if let Some(height) = tilemap.height() {
                let height = (height / tilemap.chunk_height()) as i32 / 2;
                if chunk_y < -height || chunk_y > height {
                    continue;
                }
            }

            if let Err(e) = tilemap.spawn_chunk(Point2::new(chunk_x, chunk_y)) {
                warn!("{}", e);
            }
            new_spawned.push(Point2::new(chunk_x, chunk_y));
        }
    }

    let spawned_list = tilemap.spawned_chunks_mut().clone();
    for point in spawned_list.iter() {
        if !new_spawned.contains(&point.into()) {
            if let Err(e) = tilemap.despawn_chunk(point) {
                warn!("{}", e);
            }
        }
    }
}

/// On window size change, the radius of chunks changes if needed.
pub(crate) fn tilemap_chunk_auto_radius<
    T: BaseTile,
    C: Chunk<T>,
    M: Tilemap<T, C> + WorldQuery + Component,
>(
    window_resized_events: Res<Events<WindowResized>>,
    mut tilemap_query: Query<(&mut M, &Transform)>,
    camera_query: Query<(&Camera, &Transform)>,
) {
    let mut window_reader = window_resized_events.get_reader();
    for event in window_reader.iter(&window_resized_events) {
        for (mut tilemap, tilemap_transform) in tilemap_query.iter_mut() {
            let window_width = event.width as u32;
            let window_height = event.height as u32;
            let chunk_px_width = tilemap.chunk_width() * tilemap.tile_width();
            let chunk_px_height = tilemap.chunk_height() * tilemap.tile_height();
            let chunks_wide = (window_width as f32 / chunk_px_width as f32).ceil() as u32 + 1;
            let chunks_high = (window_height as f32 / chunk_px_height as f32).ceil() as u32 + 1;
            let spawn_dimensions = Dimension2::new(chunks_wide, chunks_high);
            tilemap.set_auto_spawn(spawn_dimensions);
            for (_camera, camera_transform) in camera_query.iter() {
                auto_spawn(
                    camera_transform,
                    &tilemap_transform,
                    &mut tilemap,
                    spawn_dimensions,
                );
            }
        }
    }
}

/// Spawns and despawns chunks automatically based on a camera's position.
pub(crate) fn tilemap_chunk_auto_spawn<
    T: BaseTile,
    C: Chunk<T>,
    M: Tilemap<T, C> + WorldQuery + Component,
>(
    mut tilemap_query: Query<(&mut M, &Transform)>,
    camera_query: Query<(&Camera, &Transform), Changed<Transform>>,
) {
    // For the transform, get chunk coord.
    for (mut tilemap, tilemap_transform) in tilemap_query.iter_mut() {
        for (_camera, camera_transform) in camera_query.iter() {
            let spawn_dimensions = if let Some(dimensions) = tilemap.auto_spawn() {
                dimensions
            } else {
                continue;
            };
            auto_spawn(
                camera_transform,
                &tilemap_transform,
                &mut tilemap,
                spawn_dimensions,
            );
        }
    }
}
