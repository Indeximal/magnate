use std::{io::Write, path::PathBuf};

use bevy::{ecs::system::CommandQueue, prelude::*, utils::HashMap};

use crate::{
    tilemap::{spawn_triangle, FaceCoord, TriangleTile},
    AssetHandles, GameState,
};

pub struct MagnateLevelPlugin;

impl Plugin for MagnateLevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(GameState::Next)
                .with_system(save_system.exclusive_system())
                .with_system(load_system.exclusive_system()),
        );
    }
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
}

fn read_json(name: &str) -> Result<String, ()> {
    // from https://github.com/rparrett/pixie_wrangler/blob/main/src/save.rs
    #[cfg(not(target_arch = "wasm32"))]
    {
        match std::fs::read_to_string(json_path(name)) {
            Ok(s) => Ok(s),
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

        Ok(String::from(item))
    }
}

fn save_system(world: &mut World) {
    // Continue on Ctrl+S
    let keys = world.resource::<Input<KeyCode>>();
    if !(keys.just_pressed(KeyCode::S) && keys.pressed(KeyCode::LControl)) {
        return;
    }

    let mut query = world.query::<(&TriangleTile, &Parent)>();
    let tris = serde_json::to_string(
        &query
            .iter(world)
            .map(|(t, p)| (t.position, p.get()))
            .collect::<Vec<_>>(),
    );

    match tris {
        Ok(data) => write_json(data, "1"),
        Err(e) => warn!("Failed to serialize save file: {:?}", e),
    };
}

fn load_system(world: &mut World) {
    // Continue on 1
    let keys = world.resource::<Input<KeyCode>>();
    if !keys.just_pressed(KeyCode::Key1) {
        return;
    }

    spawn_level(world, "1");
}

pub fn spawn_level(world: &mut World, name: &str) {
    let deser = match read_json(name) {
        Ok(data) => serde_json::from_str::<Vec<(FaceCoord, Entity)>>(&data),
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
        let trig = spawn_triangle(
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
