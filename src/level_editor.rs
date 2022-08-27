use bevy::{prelude::*, render::camera::RenderTarget, sprite::MaterialMesh2dBundle};
use bevy_point_selection::{viewport_to_world, Selectable};
use rand::Rng;

use crate::{
    tilemap::{
        FromWorldPosition, Immovable, RuneTile, TileCoord, TransformInWorld, TriangleTile,
        TRIANGLE_SIDE, X_DIR, Y_DIR,
    },
    AssetHandles, GameState, SpriteAssets,
};

const SELECTABLE_RADIUS: f32 = 0.25 * TRIANGLE_SIDE;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuilderState {
    Triangles,
    Immovables,
    Runes,
}

/// Dynamically add Triangles, Immovables and Runes with a mouseclick.
/// Press `A` to select Triangles, `S` for Immovables and `D` for Runes.
/// Then hold Left Control while clicking on a tile to place it.
///
/// Use the [`crate::savegame::MagnateSaveGamePlugin`] to save the levels.
pub struct MagnateLevelEditorPlugin;

impl Plugin for MagnateLevelEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(GameState::Next)
                .with_system(builder)
                .with_system(update_builder_state),
        )
        .add_state(BuilderState::Triangles);
    }
}

fn update_builder_state(mut state: ResMut<State<BuilderState>>, keys: Res<Input<KeyCode>>) {
    let _ = if keys.just_pressed(KeyCode::A) {
        state.set(BuilderState::Triangles)
    } else if keys.just_pressed(KeyCode::S) {
        state.set(BuilderState::Immovables)
    } else if keys.just_pressed(KeyCode::D) {
        state.set(BuilderState::Runes)
    } else {
        Ok(())
    };
}

fn builder(
    commands: Commands,
    keys: Res<Input<KeyCode>>,
    mouse_btn: Res<Input<MouseButton>>,
    state: Res<State<BuilderState>>,
    sprites: Res<SpriteAssets>,
    assets: Res<AssetHandles>,
    windows: Res<Windows>,
    cam: Query<(&Camera, &GlobalTransform)>,
) {
    builder_fallable(
        commands, keys, mouse_btn, state, sprites, assets, windows, cam,
    );
}

fn builder_fallable(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    mouse_btn: Res<Input<MouseButton>>,
    state: Res<State<BuilderState>>,
    sprites: Res<SpriteAssets>,
    assets: Res<AssetHandles>,
    windows: Res<Windows>,
    cam: Query<(&Camera, &GlobalTransform)>,
) -> Option<()> {
    if !keys.pressed(KeyCode::LControl) {
        return None;
    }
    if !mouse_btn.any_just_pressed([MouseButton::Left, MouseButton::Right]) {
        return None;
    }

    let (camera, cam_transform) = cam.get_single().ok()?;
    let window_id = match camera.target {
        RenderTarget::Window(id) => id,
        _ => return None,
    };
    let window = windows.get(window_id)?;
    let cursor_position = viewport_to_world(camera, cam_transform, window)?;
    let coord = FromWorldPosition::from_world_pos(cursor_position);

    match state.current() {
        BuilderState::Triangles => {
            let tri = spawn_solo_triangle(
                &mut commands,
                coord,
                assets.triangle_mesh.clone(),
                assets.triangle_material.clone(),
            );
            commands
                .spawn()
                .insert_bundle(TransformBundle::default())
                .insert_bundle(VisibilityBundle::default())
                .add_child(tri);
        }
        BuilderState::Immovables => {
            spawn_immovable(
                &mut commands,
                coord,
                assets.triangle_mesh.clone(),
                assets.immovable_material.clone(),
            );
        }
        BuilderState::Runes => {
            spawn_rune(&mut commands, coord, sprites.runes.clone());
        }
    };

    Some(())
}

pub fn spawn_immovable(
    commands: &mut Commands,
    coord: TileCoord,
    mesh: Handle<Mesh>,
    mat: Handle<ColorMaterial>,
) -> Entity {
    let tile = TriangleTile { position: coord };
    commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: mesh.into(),
            transform: tile.to_world_pos(),
            material: mat,
            ..default()
        })
        .insert(tile)
        .insert(Immovable)
        .id()
}

pub fn spawn_solo_triangle(
    commands: &mut Commands,
    coord: TileCoord,
    mesh: Handle<Mesh>,
    mat: Handle<ColorMaterial>,
) -> Entity {
    let tile = TriangleTile { position: coord };
    commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: mesh.into(),
            transform: tile.to_world_pos(),
            material: mat,
            ..default()
        })
        .insert(tile)
        .with_children(|builder| {
            builder
                .spawn_bundle(TransformBundle::from_transform(Transform::default()))
                .insert(Selectable::new(SELECTABLE_RADIUS));
            builder
                .spawn_bundle(TransformBundle::from_transform(
                    Transform::from_translation(X_DIR.extend(0.)),
                ))
                .insert(Selectable::new(SELECTABLE_RADIUS));
            builder
                .spawn_bundle(TransformBundle::from_transform(
                    Transform::from_translation(Y_DIR.extend(0.)),
                ))
                .insert(Selectable::new(SELECTABLE_RADIUS));
        })
        .id()
}

pub fn spawn_rune(
    commands: &mut Commands,
    coord: TileCoord,
    atlas: Handle<TextureAtlas>,
) -> Entity {
    let tile = RuneTile { position: coord };

    commands
        .spawn_bundle(SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(rand::thread_rng().gen_range(0..5) * 2),
            texture_atlas: atlas,
            transform: tile.to_world_pos(),
            ..Default::default()
        })
        .insert(tile)
        .id()
}
