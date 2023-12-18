use std::fs::read_to_string;

use bevy::{
    asset::{AssetServer, Assets, Handle},
    ecs::{
        component::Component,
        entity::Entity,
        system::{Commands, NonSend, Query, Res, ResMut},
    },
    hierarchy::BuildChildren,
    math::{UVec2, Vec2},
    prelude::SpatialBundle,
    render::render_resource::FilterMode,
    sprite::{Sprite, SpriteBundle, SpriteSheetBundle, TextureAtlas, TextureAtlasSprite},
    transform::components::Transform,
    utils::{hashbrown::HashMap, HashSet},
};

use crate::render::texture::{TilemapTexture, TilemapTextureDescriptor};

use self::{
    entities::LdtkEntityRegistry,
    json::{
        definitions::{LayerType, TilesetDef},
        level::{LayerInstance, Level},
        LdtkJson, WorldLayout,
    },
    layer::LdtkLayers,
};

pub mod app_ext;
pub mod components;
pub mod entities;
pub mod enums;
pub mod json;
pub mod layer;
pub mod manager;

#[derive(Component)]
pub struct LdtkLoader {
    pub(crate) path: String,
    pub(crate) asset_path_prefix: String,
    pub(crate) level: HashSet<String>,
    pub(crate) level_spacing: Option<i32>,
    pub(crate) filter_mode: FilterMode,
    pub(crate) ignore_unregistered_entities: bool,
    pub(crate) z_index: i32,
    pub(crate) atlas_render_size: HashMap<String, Vec2>,
}

pub fn load_ldtk_json(
    mut commands: Commands,
    loader_query: Query<(Entity, &LdtkLoader)>,
    asset_server: Res<AssetServer>,
    ident_mapper: NonSend<LdtkEntityRegistry>,
    mut atlas_asstes: ResMut<Assets<TextureAtlas>>,
) {
    for (entity, loader) in loader_query.iter() {
        let path = std::env::current_dir().unwrap().join(&loader.path);
        let str_raw = match read_to_string(&path) {
            Ok(data) => data,
            Err(e) => panic!("Could not read file at path: {:?}!\n{}", path, e),
        };

        let mut ldtk_data = match serde_json::from_str::<LdtkJson>(&str_raw) {
            Ok(data) => data,
            Err(e) => panic!("Could not parse file at path: {}!\n{}", loader.path, e),
        };

        load_levels(
            &mut commands,
            &mut ldtk_data,
            loader,
            &asset_server,
            &ident_mapper,
            &mut atlas_asstes,
            entity,
        );

        commands.entity(entity).insert(SpatialBundle::default());
        commands.entity(entity).remove::<LdtkLoader>();
    }
}

fn load_levels(
    commands: &mut Commands,
    ldtk_data: &mut LdtkJson,
    loader: &LdtkLoader,
    asset_server: &AssetServer,
    ident_mapper: &LdtkEntityRegistry,
    atlas_asstes: &mut Assets<TextureAtlas>,
    level_entity: Entity,
) {
    let mut tilesets = HashMap::with_capacity(ldtk_data.defs.tilesets.len());
    let mut atlas_handles = HashMap::with_capacity(ldtk_data.defs.tilesets.len());
    let mut tileset_uid_to_ident = HashMap::with_capacity(ldtk_data.defs.tilesets.len());

    ldtk_data.defs.tilesets.iter().for_each(|tileset| {
        if let Some(texture) = load_texture(tileset, &loader, asset_server) {
            tilesets.insert(tileset.uid, texture.clone());
            tileset_uid_to_ident.insert(tileset.uid, tileset.identifier.clone());

            let handle = atlas_asstes.add(texture.as_texture_atlas());
            atlas_handles.insert(tileset.uid, handle);
        }
    });

    for (level_index, level) in ldtk_data.levels.iter().enumerate() {
        if !loader.level.contains(&level.identifier) {
            continue;
        }

        let translation = get_level_translation(&ldtk_data, loader, level_index);

        let level_px = UVec2 {
            x: level.px_wid as u32,
            y: level.px_hei as u32,
        };

        load_background(
            commands,
            level_entity,
            level,
            loader,
            translation,
            level_px,
            asset_server,
        );

        let mut layer_grid = LdtkLayers::new(
            level_entity,
            level.layer_instances.len(),
            level_px,
            &tilesets,
            translation,
            loader.z_index,
        );
        for (layer_index, layer) in level.layer_instances.iter().enumerate() {
            load_layer(
                commands,
                level_entity,
                layer_index,
                layer,
                &mut layer_grid,
                &ident_mapper,
                loader,
                asset_server,
                &tilesets,
                &atlas_handles,
                &tileset_uid_to_ident,
            );
        }

        layer_grid.apply_all(commands);

        break;
    }
}

fn load_texture(
    tileset: &TilesetDef,
    loader: &LdtkLoader,
    asset_server: &AssetServer,
) -> Option<TilemapTexture> {
    let Some(path) = tileset.rel_path.as_ref() else {
        return None;
    };

    let texture = asset_server.load(format!("{}{}", loader.asset_path_prefix, path));
    let desc = TilemapTextureDescriptor {
        size: UVec2 {
            x: tileset.px_wid as u32,
            y: tileset.px_hei as u32,
        },
        tile_size: UVec2 {
            x: tileset.tile_grid_size as u32,
            y: tileset.tile_grid_size as u32,
        },
        filter_mode: loader.filter_mode,
    };
    Some(TilemapTexture { texture, desc })
}

fn load_background(
    commands: &mut Commands,
    level_entity: Entity,
    level: &Level,
    loader: &LdtkLoader,
    translation: Vec2,
    level_px: UVec2,
    asset_server: &AssetServer,
) {
    let texture = match level.bg_rel_path.as_ref() {
        Some(path) => asset_server.load(format!("{}{}", loader.asset_path_prefix, path)),
        None => Handle::default(),
    };

    let bg_entity = commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: level.bg_color.into(),
                custom_size: Some(level_px.as_vec2()),
                ..Default::default()
            },
            texture,
            transform: Transform::from_translation(
                (translation + level_px.as_vec2() / 2.)
                    .extend(loader.z_index as f32 - level.layer_instances.len() as f32 - 1.),
            ),
            ..Default::default()
        })
        .id();
    commands.entity(level_entity).add_child(bg_entity);
}

fn load_layer(
    commands: &mut Commands,
    level_entity: Entity,
    layer_index: usize,
    layer: &LayerInstance,
    layer_grid: &mut LdtkLayers,
    ident_mapper: &LdtkEntityRegistry,
    loader: &LdtkLoader,
    asset_server: &AssetServer,
    tilesets: &HashMap<i32, TilemapTexture>,
    atlas_handles: &HashMap<i32, Handle<TextureAtlas>>,
    tileset_uid_to_ident: &HashMap<i32, String>,
) {
    match layer.ty {
        LayerType::IntGrid | LayerType::AutoLayer => {
            for tile in layer.auto_layer_tiles.iter() {
                layer_grid.set(commands, layer_index, layer, tile);
            }
        }
        LayerType::Entities => {
            for entity in layer.entity_instances.iter() {
                let phantom_entity = {
                    if let Some(m) = ident_mapper.get(&entity.identifier) {
                        m
                    } else if !loader.ignore_unregistered_entities {
                        panic!(
                            "Could not find entity type with entity identifier: {}! \
                            You need to register it using App::register_ldtk_entity::<T>() first!",
                            entity.identifier
                        );
                    } else {
                        return;
                    }
                };

                let mut new_entity = commands.spawn_empty();

                let sprite_bundle = {
                    match entity.tile.as_ref() {
                        Some(atlas) => {
                            let render_size = loader
                                .atlas_render_size
                                .get(&tileset_uid_to_ident[&atlas.tileset_uid])
                                .cloned()
                                .unwrap_or(tilesets[&atlas.tileset_uid].desc.tile_size.as_vec2());

                            let entity_rel_pos = Vec2 {
                                x: entity.world_x as f32,
                                y: -entity.world_y as f32 - render_size.y,
                            };
                            let pivot_offset = Vec2 {
                                x: render_size.x * (entity.pivot[0] - 0.5),
                                y: render_size.y * (entity.pivot[1] + 0.5),
                            };

                            let sprite_trans = (entity_rel_pos + pivot_offset)
                                .extend(loader.z_index as f32 - layer_index as f32 - 1.);

                            let tileset = tilesets.get(&atlas.tileset_uid).unwrap();
                            let index = UVec2 {
                                x: atlas.x_pos as u32 / tileset.desc.tile_size.x,
                                y: atlas.y_pos as u32 / tileset.desc.tile_size.y,
                            };

                            Some(SpriteSheetBundle {
                                sprite: TextureAtlasSprite {
                                    index: (index.y * tileset.desc.size.x + index.x) as usize,
                                    custom_size: Some(render_size.clone()),
                                    ..Default::default()
                                },
                                transform: Transform::from_translation(sprite_trans),
                                texture_atlas: atlas_handles[&atlas.tileset_uid].clone(),
                                ..Default::default()
                            })
                        }
                        None => None,
                    }
                };

                let mut fields = entity
                    .field_instances
                    .iter()
                    .map(|field| (field.identifier.clone(), field.clone()))
                    .collect();
                phantom_entity.spawn(&mut new_entity, sprite_bundle, &mut fields, asset_server);

                let new_entity = new_entity.id();
                commands.entity(level_entity).add_child(new_entity);
            }
        }
        LayerType::Tiles => {
            for tile in layer.grid_tiles.iter() {
                layer_grid.set(commands, layer_index, layer, tile);
            }
        }
    }
}

fn get_level_translation(ldtk_data: &LdtkJson, loader: &LdtkLoader, index: usize) -> Vec2 {
    let level = &ldtk_data.levels[index];
    match ldtk_data.world_layout.unwrap() {
        WorldLayout::GridVania | WorldLayout::Free => Vec2 {
            x: level.world_x as f32,
            y: (-level.world_y - level.px_hei) as f32,
        },
        WorldLayout::LinearHorizontal => {
            let mut offset = 0;
            for i in 0..index {
                offset += ldtk_data.levels[i].px_wid + loader.level_spacing.unwrap();
            }
            Vec2 {
                x: offset as f32,
                y: 0.,
            }
        }
        WorldLayout::LinearVertical => {
            let mut offset = 0;
            for i in 0..index {
                offset += ldtk_data.levels[i].px_hei + loader.level_spacing.unwrap();
            }
            Vec2 {
                x: 0.,
                y: -offset as f32,
            }
        }
    }
}