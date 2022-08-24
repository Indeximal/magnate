use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_asset_loader::prelude::*;
use bevy_point_selection::{PointSelectionPlugin, SelectionSource};
use rotation::TriangleRotationPlugin;
use tilemap::{spawn_triangle, TriangleOrientation, VertexCoord};

pub const BG_COLOR: Color = Color::rgb(0.7, 0.7, 0.7);
pub const ASPECT_RATIO: f32 = 16.0 / 9.0;

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
        .add_plugin(PointSelectionPlugin)
        .add_plugin(TriangleRotationPlugin)
        .add_system_set(
            SystemSet::on_enter(GameState::Next)
                .with_system(spawn_camera)
                .with_system(spawn_triangles),
        )
        .run();
}

/// Spawn a 2d camera with a fix heigth  in triangle units, and auto width
fn spawn_camera(mut commands: Commands) {
    commands
        .spawn_bundle(Camera2dBundle {
            projection: OrthographicProjection {
                scaling_mode: ScalingMode::FixedVertical(10.),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(SelectionSource);
}

/// Spawn some triangles
fn spawn_triangles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    spawn_triangle(
        &mut commands,
        (VertexCoord::new(0, 0), TriangleOrientation::PointingUp),
        &mut meshes,
        &mut materials,
    );
    spawn_triangle(
        &mut commands,
        (VertexCoord::new(0, 0), TriangleOrientation::PointingDown),
        &mut meshes,
        &mut materials,
    );
    spawn_triangle(
        &mut commands,
        (VertexCoord::new(1, 1), TriangleOrientation::PointingUp),
        &mut meshes,
        &mut materials,
    );
    spawn_triangle(
        &mut commands,
        (VertexCoord::new(2, -1), TriangleOrientation::PointingDown),
        &mut meshes,
        &mut materials,
    );
}
