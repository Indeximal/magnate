//! # Magnate
//! A bevy game for bevy jam 2.
//!
//! Rotate triangles to light up the runes, but beware that they're inseperarable once touching.
//!
//! ## TODO:
//! - Rune logic
//!     - next level when all done, with visual clue
//!
//! - Non Moveables at the boundaries
//!
//! - Rotation Ghost
//! - Particles?
//! - Audio?
//! - Animations?
//! - Different Colors?

use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_asset_loader::prelude::*;
use bevy_point_selection::{PointSelectionPlugin, SelectionSource};
use level::MagnateLevelPlugin;
use level_editor::MagnateLevelEditorPlugin;
use rotation::MagnateRotationPlugin;
use savegame::MagnateSaveGamePlugin;

pub const BG_COLOR: Color = Color::rgb(0.7, 0.7, 0.7);

mod level;
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
        .add_plugin(MagnateLevelPlugin)
        .add_plugin(MagnateLevelEditorPlugin)
        .add_system_set(
            SystemSet::on_enter(GameState::Next)
                .with_system(spawn_camera)
                .with_system(spawn_background),
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
