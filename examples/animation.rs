use bevy::{
    app::{App, Startup},
    asset::AssetServer,
    core_pipeline::core_2d::Camera2dBundle,
    ecs::system::{Commands, Res},
    math::{UVec2, Vec2},
    render::render_resource::FilterMode,
    DefaultPlugins,
};
use bevy_entitiles::{
    math::FillArea,
    render::{
        buffer::TileAnimation,
        texture::{TilemapTexture, TilemapTextureDescriptor},
    },
    tilemap::{
        map::TilemapBuilder,
        tile::{AnimatedTile, TileBuilder, TileType},
    },
    EntiTilesPlugin,
};
use helpers::EntiTilesDebugPlugin;

mod helpers;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EntiTilesPlugin, EntiTilesDebugPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let (tilemap_entity, mut tilemap) = TilemapBuilder::new(
        TileType::Square,
        UVec2 { x: 20, y: 20 },
        Vec2 { x: 16., y: 16. },
        "test_map".to_string(),
    )
    .with_texture(TilemapTexture::new(
        asset_server.load("test_square.png"),
        TilemapTextureDescriptor {
            size: UVec2 { x: 32, y: 32 },
            tile_size: UVec2 { x: 16, y: 16 },
            filter_mode: FilterMode::Nearest,
        },
    ))
    .build(&mut commands);

    let anim_a = tilemap.register_animation(TileAnimation::new(vec![0, 1, 2, 3], 2.));
    let anim_b = tilemap.register_animation(TileAnimation::new(vec![0, 1, 2], 3.));

    tilemap.fill_rect(
        &mut commands,
        FillArea::full(&tilemap),
        &TileBuilder::new().with_animation(AnimatedTile {
            sequence_index: anim_a,
        }),
    );

    tilemap.fill_rect(
        &mut commands,
        FillArea::new(UVec2::ZERO, Some(UVec2 { x: 10, y: 10 }), &tilemap),
        &TileBuilder::new().with_animation(AnimatedTile {
            sequence_index: anim_b,
        }),
    );

    commands.entity(tilemap_entity).insert(tilemap);
}
