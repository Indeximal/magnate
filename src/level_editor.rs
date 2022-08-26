use bevy::{prelude::*, render::camera::RenderTarget, sprite::MaterialMesh2dBundle};
use bevy_point_selection::{viewport_to_world, Selectable};
use rand::Rng;

use crate::{
    tilemap::{
        FromWorldPosition, RuneTile, TileCoord, TransformInWorld, TriangleTile, TRIANGLE_SIDE,
        X_DIR, Y_DIR,
    },
    AssetHandles, GameState, SpriteAssets,
};

const SELECTABLE_RADIUS: f32 = 0.25 * TRIANGLE_SIDE;

pub struct MagnateLevelEditorPlugin;

impl Plugin for MagnateLevelEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_update(GameState::Next).with_system(builder));
    }
}

fn builder(
    commands: Commands,
    keys: Res<Input<KeyCode>>,
    mouse_btn: Res<Input<MouseButton>>,
    sprites: Res<SpriteAssets>,
    assets: Res<AssetHandles>,
    windows: Res<Windows>,
    cam: Query<(&Camera, &GlobalTransform)>,
) {
    builder_fallable(commands, keys, mouse_btn, sprites, assets, windows, cam);
}

fn builder_fallable(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    mouse_btn: Res<Input<MouseButton>>,
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

    if mouse_btn.just_pressed(MouseButton::Right) {
        spawn_rune(&mut commands, coord, sprites.runes.clone());
    } else if mouse_btn.just_pressed(MouseButton::Left) {
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

    Some(())
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
            sprite: TextureAtlasSprite::new(rand::thread_rng().gen_range(0..10)),
            texture_atlas: atlas,
            transform: tile.to_world_pos(),
            ..Default::default()
        })
        .insert(tile)
        .id()
}
