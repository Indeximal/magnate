//! # Magnate
//! A bevy game for bevy jam 2.
//!
//! Rotate triangles to light up the runes, but beware that they're inseperarable once touching.
//!
//! ## TODO:
//! - Rune logic
//!     - light up then rotated into
//!     - next level when all done
//!
//! - Non Moveables at the boundaries
//!
//! - Rotation Ghost
//! - Particles?
//! - Different Colors?

use bevy::{
    prelude::*,
    render::{camera::ScalingMode, mesh::Indices, render_resource::PrimitiveTopology},
};
use bevy_asset_loader::prelude::*;
use bevy_point_selection::{PointSelectionPlugin, SelectionSource};
use level_editor::MagnateLevelEditorPlugin;
use rotation::MagnateRotationPlugin;
use savegame::{spawn_level, LevelInfo, MagnateSaveGamePlugin};
use tilemap::{SQRT3_HALF, TRIANGLE_SIDE};

pub const BG_COLOR: Color = Color::rgb(0.7, 0.7, 0.7);

mod level_editor;
mod rotation;
mod savegame;
mod tilemap;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    AssetLoading,
    Next,
}

#[derive(AssetCollection)]
struct SpriteAssets {
    #[asset(path = "circle.png")]
    circle: Handle<Image>,
    #[asset(path = "background.png")]
    background: Handle<Image>,
    #[asset(path = "ruby_triangle.png")]
    ruby_triangle: Handle<Image>,
    #[asset(texture_atlas(
        tile_size_x = 128.,
        tile_size_y = 128.,
        columns = 2,
        rows = 5,
        padding_x = 0.,
        padding_y = 0.
    ))]
    #[asset(path = "rune_sheet.png")]
    runes: Handle<TextureAtlas>,
}

struct AssetHandles {
    triangle_mesh: Handle<Mesh>,
    triangle_material: Handle<ColorMaterial>,
}

fn main() {
    App::new()
        .insert_resource(ClearColor(BG_COLOR))
        .insert_resource(WindowDescriptor {
            width: 1200.0,
            height: 720.0,
            title: "Magnate".to_string(),
            present_mode: bevy::window::PresentMode::Fifo,
            resizable: true,
            ..Default::default()
        })
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading)
                .continue_to_state(GameState::Next)
                .with_collection::<SpriteAssets>(),
        )
        .add_state(GameState::AssetLoading)
        .add_plugins(DefaultPlugins)
        .add_plugin(PointSelectionPlugin)
        .add_plugin(MagnateRotationPlugin)
        .add_plugin(MagnateSaveGamePlugin)
        .add_plugin(MagnateLevelEditorPlugin)
        .add_system_set(
            SystemSet::on_enter(GameState::Next)
                .with_system(spawn_camera)
                .with_system(spawn_background)
                .with_system(initial_load.exclusive_system()),
        )
        .run();
}

/// Spawn a 2d camera with a fix heigth  in triangle units, and auto width
fn spawn_camera(mut commands: Commands) {
    commands
        .spawn_bundle(Camera2dBundle {
            projection: OrthographicProjection {
                scaling_mode: ScalingMode::FixedVertical(720.),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(SelectionSource);
}

/// Spawn the 1280x720 background sprite with the triangle grid
fn spawn_background(mut commands: Commands, assets: Res<SpriteAssets>) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: assets.background.clone(),
            transform: Transform::from_xyz(0.0, 0.0, 100.0),
            ..Default::default()
        })
        .insert(Name::new("Background"));
}

/// Spawn the first level
fn initial_load(world: &mut World) {
    // maybe use asset loader lib?
    //  see https://github.com/NiklasEi/bevy_asset_loader/blob/main/bevy_asset_loader/examples/custom_dynamic_assets.rs
    let sprite = world.resource::<SpriteAssets>().ruby_triangle.clone();
    let meshes = world
        .resource_mut::<Assets<Mesh>>()
        .add(create_triangle_mesh(TRIANGLE_SIDE));
    let materials = world
        .resource_mut::<Assets<ColorMaterial>>()
        .add(ColorMaterial {
            color: Color::WHITE,
            texture: Some(sprite),
        });
    let assets = AssetHandles {
        triangle_mesh: meshes,
        triangle_material: materials,
    };
    // This needs to happen before spawn_level
    world.insert_resource(assets);

    // Get the default level from [`LevelInfo`]
    let lvl = world.resource::<LevelInfo>().current;
    spawn_level(world, lvl.to_string().as_str());
}

/// create a mesh for a flippable triangle. The two sides use UV 0..0.5 and 0.5..1.
fn create_triangle_mesh(size: f32) -> Mesh {
    // pos  , normal  , uv
    // x y z, nx ny nz, u v
    let vertices = [
        ([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 0.5]),
        ([size, 0.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.5]),
        (
            [size / 2., size * SQRT3_HALF, 0.0],
            [0.0, 0.0, 1.0],
            [0.5, 0.0],
        ),
        (
            [size / 2., size * SQRT3_HALF, 0.0],
            [0.0, 0.0, 1.0],
            [0.5, 1.0],
        ),
    ];
    let indices = Indices::U32(vec![0, 1, 2, 0, 3, 1]);

    let positions: Vec<_> = vertices.iter().map(|(p, _, _)| *p).collect();
    let normals: Vec<_> = vertices.iter().map(|(_, n, _)| *n).collect();
    let uvs: Vec<_> = vertices.iter().map(|(_, _, uv)| *uv).collect();

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(indices));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh
}
