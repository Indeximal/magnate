#[cfg(not(target_arch = "wasm32"))]
use std::{io::Write, path::PathBuf};

use bevy::{
    ecs::system::CommandQueue, prelude::*, render::camera::RenderTarget,
    sprite::MaterialMesh2dBundle, utils::HashMap,
};
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

const LEVELS: &[&'static str] = &[
    include_str!("../levels/1.json"), // TODO Create level 0
    include_str!("../levels/1.json"),
];

pub struct LevelInfo {
    pub current: usize,
}

impl Default for LevelInfo {
    fn default() -> Self {
        Self { current: 1 }
    }
}

pub struct MagnateLevelPlugin;

impl Plugin for MagnateLevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(GameState::Next)
                .with_system(save_system.exclusive_system())
                .with_system(load_system.exclusive_system())
                .with_system(rune_builder),
        )
        .init_resource::<LevelInfo>();
    }
}

fn rune_builder(
    commands: Commands,
    keys: Res<Input<KeyCode>>,
    mouse_btn: Res<Input<MouseButton>>,
    sprites: Res<SpriteAssets>,
    windows: Res<Windows>,
    cam: Query<(&Camera, &GlobalTransform)>,
) {
    rune_builder_fallable(commands, keys, mouse_btn, sprites, windows, cam);
}

fn rune_builder_fallable(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    mouse_btn: Res<Input<MouseButton>>,
    sprites: Res<SpriteAssets>,
    windows: Res<Windows>,
    cam: Query<(&Camera, &GlobalTransform)>,
) -> Option<()> {
    if !keys.pressed(KeyCode::LControl) {
        return None;
    }
    if !mouse_btn.just_pressed(MouseButton::Left) {
        return None;
    }

    let (camera, cam_transform) = cam.get_single().ok()?;
    let window_id = match camera.target {
        RenderTarget::Window(id) => id,
        _ => return None,
    };
    let window = windows.get(window_id)?;
    let cursor_position = viewport_to_world(camera, cam_transform, window)?;

    let tile = RuneTile {
        position: FromWorldPosition::from_world_pos(cursor_position),
    };

    commands.spawn_bundle(SpriteSheetBundle {
        sprite: TextureAtlasSprite::new(rand::thread_rng().gen_range(0..10)),
        texture_atlas: sprites.runes.clone(),
        transform: tile.to_world_pos(),
        ..Default::default()
    });

    Some(())
}

fn spawn_solo_triangle(
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

#[cfg(not(target_arch = "wasm32"))]
fn json_path(name: &str) -> PathBuf {
    std::path::Path::new("levels")
        .join(name)
        .with_extension("json")
}

fn write_json(data: String, name: &str) {
    // from https://github.com/rparrett/pixie_wrangler/blob/main/src/save.rs
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut file = match std::fs::File::create(json_path(name)) {
            Ok(f) => f,
            Err(e) => {
                warn!("Failed to create save file: {:?}", e);
                return;
            }
        };

        if let Err(e) = file.write(data.as_bytes()) {
            warn!("Failed to write save data: {:?}", e);
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        let window = match web_sys::window() {
            Some(w) => w,
            None => return,
        };

        let storage = match window.local_storage() {
            Ok(Some(s)) => s,
            _ => return,
        };

        if let Err(e) = storage.set_item(name, data.as_str()) {
            warn!("Failed to store save file: {:?}", e);
        }
    }

    info!("Wrote to save file {}", name);
}

fn read_json(name: &str) -> Result<String, ()> {
    // Read static levels if existing. They have the numberic names starting from "0".
    let as_num: Result<usize, _> = name.parse();
    if let Ok(i) = as_num {
        if let Some(data) = LEVELS.get(i) {
            info!("Read static save state {}", name);
            return Ok(String::from(*data));
        }
    }

    // from https://github.com/rparrett/pixie_wrangler/blob/main/src/save.rs
    #[cfg(not(target_arch = "wasm32"))]
    {
        match std::fs::read_to_string(json_path(name)) {
            Ok(s) => {
                info!("Read from save file {}", name);
                Ok(s)
            }
            Err(_) => Err(()),
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        let window = match web_sys::window() {
            Some(w) => w,
            None => return Err(()),
        };

        let storage = match window.local_storage() {
            Ok(Some(s)) => s,
            _ => return Err(()),
        };

        let item = match storage.get_item(name) {
            Ok(Some(i)) => i,
            _ => return Err(()),
        };
        info!("Read from save state {}", name);
        Ok(String::from(item))
    }
}

fn save_system(world: &mut World) {
    let keys = world.resource::<Input<KeyCode>>();
    let is_modifier_down = keys.pressed(KeyCode::LControl);
    if !is_modifier_down {
        return;
    }
    let save_as_level = get_just_pressed_num(keys);

    let level_name = match save_as_level {
        Some(i) => i,
        None => return,
    };

    let mut query = world.query::<(&TriangleTile, &Parent)>();
    let tris = serde_json::to_string(
        &query
            .iter(world)
            .map(|(t, p)| (t.position, p.get()))
            .collect::<Vec<_>>(),
    );

    match tris {
        Ok(data) => write_json(data, level_name.to_string().as_str()),
        Err(e) => warn!("Failed to serialize save file: {:?}", e),
    };
}

/// Load Levels when pressing either the number buttons for a specific level or R to restart the level
fn load_system(world: &mut World) {
    let keys = world.resource::<Input<KeyCode>>();
    let is_modifier_down = keys.pressed(KeyCode::LControl);
    if is_modifier_down {
        return;
    }
    let jump_to_level_key = get_just_pressed_num(keys);
    let should_reload = keys.just_pressed(KeyCode::R);

    let mut lvl = world.resource_mut::<LevelInfo>();

    if let Some(key) = jump_to_level_key {
        lvl.current = key;
        spawn_level(world, key.to_string().as_str());
    } else if should_reload {
        let curr = lvl.current;
        spawn_level(world, curr.to_string().as_str());
    }
}

/// Replaces the world content with the level named `name`. Numerical names are the
/// prebuilt levels.
pub fn spawn_level(world: &mut World, name: &str) {
    let deser = match read_json(name) {
        Ok(data) => serde_json::from_str::<Vec<(TileCoord, Entity)>>(&data),
        Err(_) => {
            warn!("Failed to read save file: {}", name);
            return;
        }
    };

    let data = match deser {
        Ok(tris) => tris,
        Err(e) => {
            warn!("Failed to deserialize save file: {:?}", e);
            return;
        }
    };

    clear_world(world);
    let assets = world.resource::<AssetHandles>();

    let mut command_queue = CommandQueue::default();
    let mut commands = Commands::new(&mut command_queue, world);
    // old clump id mapped to new triangle ids
    let mut clumps: HashMap<Entity, Vec<Entity>> = HashMap::new();

    for (coord, old_clump_id) in data {
        let trig = spawn_solo_triangle(
            &mut commands,
            coord,
            assets.triangle_mesh.clone(),
            assets.triangle_material.clone(),
        );

        match clumps.get_mut(&old_clump_id) {
            Some(v) => v.push(trig),
            None => {
                let _ = clumps.insert(old_clump_id, vec![trig]);
            }
        };
    }

    for (_, children) in clumps {
        commands
            .spawn()
            .insert_bundle(TransformBundle::default())
            .insert_bundle(VisibilityBundle::default())
            .push_children(&children);
    }

    command_queue.apply(world);
}

pub fn clear_world(world: &mut World) {
    let mut current_tris = world.query_filtered::<&Parent, With<TriangleTile>>();
    let current_clumps: Vec<Entity> = current_tris.iter(world).map(|p| p.get()).collect();
    for clump in current_clumps {
        despawn_with_children_recursive(world, clump);
    }
}

fn get_just_pressed_num(keys: &Input<KeyCode>) -> Option<usize> {
    // UUUGGGLYYYY
    if keys.just_pressed(KeyCode::Key0) {
        return Some(0);
    }
    if keys.just_pressed(KeyCode::Key1) {
        return Some(1);
    }
    if keys.just_pressed(KeyCode::Key2) {
        return Some(2);
    }
    if keys.just_pressed(KeyCode::Key3) {
        return Some(3);
    }
    if keys.just_pressed(KeyCode::Key4) {
        return Some(4);
    }
    if keys.just_pressed(KeyCode::Key5) {
        return Some(5);
    }
    if keys.just_pressed(KeyCode::Key6) {
        return Some(6);
    }
    if keys.just_pressed(KeyCode::Key7) {
        return Some(7);
    }
    if keys.just_pressed(KeyCode::Key8) {
        return Some(8);
    }
    if keys.just_pressed(KeyCode::Key9) {
        return Some(9);
    }
    None
}
