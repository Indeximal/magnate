//! # Magnate
//! A bevy game for bevy jam 2.
//!
//! Rotate triangles to light up the glyphs, but beware that they're inseperarable once touching.
//!
//! ## TODO:
//! - Level Editor, ie saving/loading scenes
//! - Goal glyphs and check
//!
//! - Rotation Ghost
//! - Particles?
//!
//! - Different Colors
//! - Non Moveables

use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_asset_loader::prelude::*;
use bevy_point_selection::{PointSelectionPlugin, SelectionSource};
use rotation::TriangleRotationPlugin;
use tilemap::{
    create_triangle, spawn_triangle, IterNeighbors, TriangleOrient, VertexCoord, TRIANGLE_SIDE,
};

pub const BG_COLOR: Color = Color::rgb(0.7, 0.7, 0.7);

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
        .add_plugin(TriangleRotationPlugin)
        .add_system_set(
            SystemSet::on_enter(GameState::Next)
                .with_system(spawn_camera)
                .with_system(spawn_triangles)
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

/// Spawn some triangles
fn spawn_triangles(
    mut commands: Commands,
    sprites: Res<SpriteAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // maybe use asset loader lib?
    //  see https://github.com/NiklasEi/bevy_asset_loader/blob/main/bevy_asset_loader/examples/custom_dynamic_assets.rs
    let assets = AssetHandles {
        triangle_mesh: meshes.add(create_triangle(TRIANGLE_SIDE)),
        triangle_material: materials.add(ColorMaterial {
            color: Color::WHITE,
            texture: Some(sprites.ruby_triangle.clone()),
        }),
    };

    // large triangle down
    let p1 = (VertexCoord::new(0, 0), TriangleOrient::PointingUp);
    let tri1 = spawn_triangle(
        &mut commands,
        p1,
        assets.triangle_mesh.clone(),
        assets.triangle_material.clone(),
    );

    // let tri1_neighbors: Vec<Entity> = p1
    //     .iter_neighbors()
    //     .map(|p| spawn_triangle(&mut commands, p, assets.triangle_mesh.clone(), assets.triangle_material.clone()))
    //     .collect();
    commands
        .spawn_bundle(TransformBundle::default())
        .insert_bundle(VisibilityBundle::default())
        .add_child(tri1);
    // .push_children(tri1_neighbors.as_slice());

    // large triangle up
    let p2 = (VertexCoord::new(-4, 0), TriangleOrient::PointingDown);
    let tri2 = spawn_triangle(
        &mut commands,
        p2,
        assets.triangle_mesh.clone(),
        assets.triangle_material.clone(),
    );

    let tri2_neighbors: Vec<Entity> = p2
        .iter_neighbors()
        .map(|p| {
            spawn_triangle(
                &mut commands,
                p,
                assets.triangle_mesh.clone(),
                assets.triangle_material.clone(),
            )
        })
        .collect();
    commands
        .spawn_bundle(TransformBundle::default())
        .insert_bundle(VisibilityBundle::default())
        .add_child(tri2)
        .push_children(tri2_neighbors.as_slice());

    // single triangle, bottom right corner
    let tri4 = spawn_triangle(
        &mut commands,
        (VertexCoord::new(6, -3), TriangleOrient::PointingDown),
        assets.triangle_mesh.clone(),
        assets.triangle_material.clone(),
    );
    commands
        .spawn_bundle(TransformBundle::default())
        .insert_bundle(VisibilityBundle::default())
        .add_child(tri4);

    // single triangle top left
    let tri4 = spawn_triangle(
        &mut commands,
        (VertexCoord::new(-8, 4), TriangleOrient::PointingUp),
        assets.triangle_mesh.clone(),
        assets.triangle_material.clone(),
    );
    commands
        .spawn_bundle(TransformBundle::default())
        .insert_bundle(VisibilityBundle::default())
        .add_child(tri4);

    // single triangle, bottom left
    let tri4 = spawn_triangle(
        &mut commands,
        (VertexCoord::new(-4, -4), TriangleOrient::PointingUp),
        assets.triangle_mesh.clone(),
        assets.triangle_material.clone(),
    );
    commands
        .spawn_bundle(TransformBundle::default())
        .insert_bundle(VisibilityBundle::default())
        .add_child(tri4);

    commands.insert_resource(assets);
}
