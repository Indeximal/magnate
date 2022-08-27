#[cfg(not(target_arch = "wasm32"))]
use std::{io::Write, path::PathBuf};

use bevy::{ecs::system::CommandQueue, prelude::*, utils::HashMap};

use serde::{Deserialize, Serialize};

use crate::{
    level::{LevelInfo, ReloadHint, SoftDespawned},
    level_editor::{spawn_immovable, spawn_rune, spawn_solo_triangle},
    tilemap::{Immovable, RuneTile, TileCoord, TriangleTile},
    AssetHandles, GameState, SpriteAssets,
};

const LEVELS: &[&'static str] = &[
    include_str!("../levels/0.json"),   // Level 0 is empty
    include_str!("../levels/1.json"),   // This is the first tutorial level
    include_str!("../levels/2.json"),   // This is the level with the wall
    include_str!("../levels/3.json"),   // This is the level where you can't merge
    include_str!("../levels/4.json"),   // This is the level with the hexagon
    include_str!("../levels/5.json"),   // This is the level with the hole in the wall
    include_str!("../levels/end.json"), // This is the end screen
];

/// Save and load levels on the fly.
/// Press a number `0`-`9` to load a level.
/// Press `Left Control` + a number `0`-`9` to save as a level.
///
/// Note: If a level is built-in, then loading a level will always load the built-in level
/// and not the saved one. Built-in Level 0 is garanteed to be empty, the game starts with
/// level 1.
///
/// On PC the levels are saved and loaded from `./levels`. On the web the are stored
/// in `LocalStorage`.
pub struct MagnateSaveGamePlugin;

impl Plugin for MagnateSaveGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(GameState::Next)
                .with_system(save_system.exclusive_system())
                .with_system(load_system.exclusive_system()),
        )
        .init_resource::<LevelInfo>();
    }
}

#[derive(Serialize, Deserialize)]
struct SaveGame {
    triangles: Vec<(TriangleTile, Entity)>,
    immovables: Vec<TileCoord>,
    runes: Vec<RuneTile>,
}

pub fn save_level(world: &mut World, as_name: &str) {
    // Serialize level data
    let mut tris_query = world.query::<(&TriangleTile, &Parent)>();
    let triangles = tris_query
        .iter(world)
        .map(|(t, p)| (t.clone(), p.get()))
        .collect::<Vec<(TriangleTile, Entity)>>();

    let mut immov_query = world.query_filtered::<&TriangleTile, With<Immovable>>();
    let immovables = immov_query
        .iter(world)
        .map(|t| t.position)
        .collect::<Vec<TileCoord>>();

    let mut runes_query = world.query::<&RuneTile>();
    let runes = runes_query
        .iter(world)
        .map(|t| t.clone())
        .collect::<Vec<RuneTile>>();

    let save = SaveGame {
        triangles,
        runes,
        immovables,
    };

    let ser = serde_json::to_string(&save);

    match ser {
        Ok(data) => write_json(data, as_name.to_string().as_str()),
        Err(e) => warn!("Failed to serialize save file: {:?}", e),
    };
}

/// Replaces the world content with the level named `name`. Numerical names are the
/// prebuilt levels.
pub fn spawn_level(world: &mut World, name: &str) {
    let deser = match read_json(name) {
        Ok(data) => serde_json::from_str::<SaveGame>(&data),
        Err(_) => {
            warn!("Failed to read save file: {}", name);
            return;
        }
    };

    let save = match deser {
        Ok(data) => data,
        Err(e) => {
            warn!("Failed to deserialize save file: {:?}", e);
            return;
        }
    };

    clear_world(world);

    // Spawn level data
    let assets = world.resource::<AssetHandles>();

    let mut command_queue = CommandQueue::default();
    let mut commands = Commands::new(&mut command_queue, world);
    // old clump id mapped to new triangle ids
    let mut clumps: HashMap<Entity, Vec<Entity>> = HashMap::new();

    // Spawn triangles
    for (tile, old_clump_id) in save.triangles {
        let trig = spawn_solo_triangle(
            &mut commands,
            tile.position,
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

    // Spawn triangle clump parents
    for (_, children) in clumps {
        commands
            .spawn()
            .insert_bundle(TransformBundle::default())
            .insert_bundle(VisibilityBundle::default())
            .push_children(&children);
    }

    // Spawn immovables
    for coord in save.immovables {
        spawn_immovable(
            &mut commands,
            coord,
            assets.triangle_mesh.clone(),
            assets.immovable_material.clone(),
        );
    }

    // Spawn runes
    let sprites = world.resource::<SpriteAssets>();
    for rune in save.runes {
        spawn_rune(&mut commands, rune.position, sprites.runes.clone());
    }

    command_queue.apply(world);
}

pub fn clear_world(world: &mut World) {
    let mut current_tris = world.query_filtered::<&Parent, With<TriangleTile>>();
    // Collection is necessary to prevent concurrent modification
    let current_clumps: Vec<Entity> = current_tris.iter(world).map(|p| p.get()).collect();
    for clump in current_clumps {
        despawn_with_children_recursive(world, clump);
    }

    let mut current_immovables = world.query_filtered::<Entity, With<Immovable>>();
    let current_immovables: Vec<Entity> = current_immovables.iter(world).collect();
    for immovable in current_immovables {
        despawn_with_children_recursive(world, immovable);
    }

    let mut current_runes = world.query_filtered::<Entity, With<RuneTile>>();
    let current_runes: Vec<Entity> = current_runes.iter(world).collect();
    for rune in current_runes {
        despawn_with_children_recursive(world, rune);
    }
}

/// System to load levels when pressing either the number buttons for a specific level
/// or R to restart the current one.
fn load_system(world: &mut World) {
    let keys = world.resource::<Input<KeyCode>>();
    let is_modifier_down = keys.pressed(KeyCode::LControl);
    if is_modifier_down {
        // dont load when saving
        return;
    }
    let jump_to_level_key = get_just_pressed_num(keys);
    let manual_reload = keys.just_pressed(KeyCode::R);

    let mut lvl = world.resource_mut::<LevelInfo>();
    let next_level_reload = lvl.should_reload;

    if let Some(key) = jump_to_level_key {
        lvl.current = key;
        spawn_level(world, key.to_string().as_str());
    } else if next_level_reload || manual_reload {
        let curr = lvl.current;
        spawn_level(world, curr.to_string().as_str());

        if manual_reload {
            // Remove hint
            let mut hint_query =
                world.query_filtered::<Entity, (With<ReloadHint>, Without<SoftDespawned>)>();
            let time = world.resource::<Time>().time_since_startup();
            if let Ok(id) = hint_query.get_single(&world) {
                world
                    .entity_mut(id)
                    .insert(SoftDespawned { death_time: time });
            }
        }

        // Reset
        world.resource_mut::<LevelInfo>().should_reload = false;
    }
}

/// System to save the current state when pressing CTRL + a number button.
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

    save_level(world, level_name.to_string().as_str());
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
