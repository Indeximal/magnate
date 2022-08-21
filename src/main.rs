use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_asset_loader::prelude::*;

pub const BG_COLOR: Color = Color::rgb(0.7, 0.7, 0.7);
pub const ASPECT_RATIO: f32 = 16.0 / 9.0;
pub const SQRT3_HALF: f32 = 0.866025404;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    AssetLoading,
    Next,
}

#[derive(AssetCollection)]
struct SpriteAssets {
    #[asset(path = "triangle.png")]
    triangle: Handle<Image>,
}

fn main() {
    let height = 900.0;
    App::new()
        .insert_resource(ClearColor(BG_COLOR))
        .insert_resource(WindowDescriptor {
            width: height * ASPECT_RATIO,
            height: height,
            title: "Magnate".to_string(),
            present_mode: bevy::window::PresentMode::Fifo,
            resizable: false,
            ..Default::default()
        })
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading)
                .continue_to_state(GameState::Next)
                .with_collection::<SpriteAssets>(),
        )
        .add_state(GameState::AssetLoading)
        .add_plugins(DefaultPlugins)
        .add_system_set(SystemSet::on_enter(GameState::Next).with_system(spawn_camera))
        .add_system_set(SystemSet::on_enter(GameState::Next).with_system(spawn_triangle))
        .run();
}

/// Spawn a 2d camera with a heigth of 100 units, and auto width
fn spawn_camera(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle {
        projection: OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical(100.),
            ..Default::default()
        },
        ..Default::default()
    });
}

/// Spawn a sprite with a triangle image
fn spawn_triangle(mut commands: Commands, assets: Res<SpriteAssets>) {
    commands.spawn_bundle(SpriteBundle {
        texture: assets.triangle.clone(),
        sprite: Sprite {
            custom_size: Some(Vec2::new(1., SQRT3_HALF) * 50.),
            ..Default::default()
        },
        ..Default::default()
    });
}
