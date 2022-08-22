use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_asset_loader::prelude::*;
use bevy_point_selection::{PointSelectionPlugin, Selectable, SelectionSource};
use tilemap::{spawn_triangle, TriangleOrientation, VertexCoord};

pub const BG_COLOR: Color = Color::rgb(0.7, 0.7, 0.7);
pub const ASPECT_RATIO: f32 = 16.0 / 9.0;

mod tilemap;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    AssetLoading,
    Next,
}

#[derive(AssetCollection)]
struct SpriteAssets {
    #[asset(path = "triangle.png")]
    triangle: Handle<Image>,
    sth: Handle<Mesh>,
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
        .add_system_set(SystemSet::on_enter(GameState::Next).with_system(spawn_camera))
        .add_system_set(SystemSet::on_enter(GameState::Next).with_system(spawn_triangles))
        .add_system(foo)
        .run();
}

/// Spawn a 2d camera with a heigth of 15 units, and auto width
fn spawn_camera(mut commands: Commands) {
    commands
        .spawn_bundle(Camera2dBundle {
            projection: OrthographicProjection {
                scaling_mode: ScalingMode::FixedVertical(15.),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(SelectionSource);
}

/// Spawn a sprite with a triangle image
fn spawn_triangles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    spawn_triangle(
        &mut commands,
        (VertexCoord::new(0, 0, 0), TriangleOrientation::PointingUp),
        &mut meshes,
        &mut materials,
    );
    spawn_triangle(
        &mut commands,
        (VertexCoord::new(0, 0, 0), TriangleOrientation::PointingDown),
        &mut meshes,
        &mut materials,
    );
}

fn foo(triggers: Query<&Selectable, Changed<Selectable>>) {
    for x in triggers.iter() {
        info!("changed {}", x.is_selected);
    }
}
