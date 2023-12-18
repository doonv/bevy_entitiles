use bevy::{
    ecs::entity::Entity,
    math::{UVec2, Vec2, Vec4},
};

use crate::{
    math::aabb::AabbBox2d,
    render::{buffer::TileAnimation, extract::ExtractedTilemap, texture::TilemapTexture},
    tilemap::{map::Tilemap, tile::TileType},
    MAX_ANIM_COUNT,
};

pub struct PubTilemap {
    pub id: Entity,
    pub tile_type: TileType,
    pub ext_dir: Vec2,
    pub size: UVec2,
    pub tile_render_size: Vec2,
    pub tile_slot_size: Vec2,
    pub pivot: Vec2,
    pub render_chunk_size: u32,
    pub texture: Option<TilemapTexture>,
    pub layer_opacities: Vec4,
    pub tiles: Vec<Option<Entity>>,
    pub aabb: AabbBox2d,
    pub translation: Vec2,
    pub z_index: i32,
    pub anim_seqs: [TileAnimation; MAX_ANIM_COUNT],
}

impl PubTilemap {
    pub fn from_tilemap(value: &Tilemap) -> Self {
        Self {
            id: value.id,
            tile_type: value.tile_type,
            ext_dir: value.ext_dir,
            size: value.size,
            tile_render_size: value.tile_render_size,
            tile_slot_size: value.tile_slot_size,
            pivot: value.pivot,
            render_chunk_size: value.render_chunk_size,
            texture: value.texture.clone(),
            layer_opacities: value.layer_opacities,
            tiles: value.tiles.clone(),
            aabb: value.aabb,
            translation: value.translation,
            z_index: value.z_index,
            anim_seqs: value.anim_seqs,
        }
    }

    pub fn from_extracted_tilemap(value: ExtractedTilemap) -> Self {
        Self {
            id: value.id,
            tile_type: value.tile_type,
            ext_dir: value.ext_dir,
            size: value.size,
            tile_render_size: value.tile_render_size,
            tile_slot_size: value.tile_slot_size,
            pivot: value.pivot,
            render_chunk_size: value.render_chunk_size,
            texture: value.texture,
            layer_opacities: value.layer_opacities,
            tiles: vec![],
            aabb: value.aabb,
            translation: value.translation,
            z_index: value.z_index,
            anim_seqs: value.anim_seqs,
        }
    }

    pub fn into_extracted_tilemap(self) -> ExtractedTilemap {
        ExtractedTilemap {
            id: self.id,
            tile_type: self.tile_type,
            ext_dir: self.ext_dir,
            size: self.size,
            tile_render_size: self.tile_render_size,
            tile_slot_size: self.tile_slot_size,
            pivot: self.pivot,
            render_chunk_size: self.render_chunk_size,
            texture: self.texture,
            layer_opacities: self.layer_opacities,
            translation: self.translation,
            aabb: self.aabb,
            z_index: self.z_index,
            anim_seqs: self.anim_seqs,
            time: 0.,
        }
    }
}