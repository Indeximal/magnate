//! # Magnate
//! A bevy game for bevy jam 2.
//!
//! Rotate triangles to light up the glyphs, but beware that they're inseperarable once touching.
//!
//! ## TODO:
//! - Level Editor
//! - Goal glyphs and check
//! - Wasm (check save/load)
//!
//! - Rotation Ghost
//! - Particles?
//!
//! - Different Colors
//! - Non Moveables

use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_asset_loader::prelude::*;
use bevy_point_selection::{PointSelectionPlugin, SelectionSource};
use level::{spawn_level, MagnateLevelPlugin};
use rotation::MagnateRotationPlugin;
use tilemap::{create_triangle, TRIANGLE_SIDE};

pub const BG_COLOR: Color = Color::rgb(0.7, 0.7, 0.7);

mod level;
mod rotation;
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
        .add_plugin(MagnateLevelPlugin)
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
        .add(create_triangle(TRIANGLE_SIDE));
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

    spawn_level(world, "1");
}
