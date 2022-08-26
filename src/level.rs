use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    utils::HashSet,
};

use crate::{
    savegame::spawn_level,
    tilemap::{RuneTile, TileCoord, TriangleTile, SQRT3_HALF, TRIANGLE_SIDE},
    AssetHandles, GameState, SpriteAssets,
};

pub struct MagnateLevelPlugin;

impl Plugin for MagnateLevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_update(GameState::Next).with_system(rune_system))
            .add_system_set(
                SystemSet::on_enter(GameState::Next).with_system(initial_load.exclusive_system()),
            )
            .init_resource::<LevelInfo>();
    }
}

pub struct LevelInfo {
    pub current: usize,
}

impl Default for LevelInfo {
    fn default() -> Self {
        Self { current: 1 }
    }
}

fn rune_system(
    mut runes: Query<(&RuneTile, &mut TextureAtlasSprite)>,
    added_runes: Query<Entity, Added<RuneTile>>,
    changed_triangles: Query<Entity, Changed<TriangleTile>>,
    all_triangles: Query<&TriangleTile>,
) {
    if changed_triangles.is_empty() && added_runes.is_empty() {
        return;
    }
    let all_triangles: HashSet<TileCoord> = all_triangles.iter().map(|tri| tri.position).collect();

    let mut total_runes = 0;
    let mut fulfilled_runes = 0;
    for (rune, mut sprite) in runes.iter_mut() {
        if all_triangles.contains(&rune.position) {
            fulfilled_runes += 1;
            // round to odd
            sprite.index = (sprite.index / 2) * 2 + 1;
        } else {
            // round to even
            sprite.index = (sprite.index / 2) * 2;
        }
        total_runes += 1;
    }

    if total_runes > 0 && total_runes == fulfilled_runes {
        info!("You've won!");
    }
}

/// Spawn the first level
fn initial_load(world: &mut World) {
    // maybe use asset loader lib?
    //  see https://github.com/NiklasEi/bevy_asset_loader/blob/main/bevy_asset_loader/examples/custom_dynamic_assets.rs
    let sprite = world.resource::<SpriteAssets>().ruby_triangle.clone();
    let meshes = world
        .resource_mut::<Assets<Mesh>>()
        .add(create_triangle_mesh(TRIANGLE_SIDE));
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

    // Get the default level from [`LevelInfo`]
    let lvl = world.resource::<LevelInfo>().current;
    spawn_level(world, lvl.to_string().as_str());
}

/// create a mesh for a flippable triangle. The two sides use UV 0..0.5 and 0.5..1.
fn create_triangle_mesh(size: f32) -> Mesh {
    // pos  , normal  , uv
    // x y z, nx ny nz, u v
    let vertices = [
        ([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 0.5]),
        ([size, 0.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.5]),
        (
            [size / 2., size * SQRT3_HALF, 0.0],
            [0.0, 0.0, 1.0],
            [0.5, 0.0],
        ),
        (
            [size / 2., size * SQRT3_HALF, 0.0],
            [0.0, 0.0, 1.0],
            [0.5, 1.0],
        ),
    ];
    let indices = Indices::U32(vec![0, 1, 2, 0, 3, 1]);

    let positions: Vec<_> = vertices.iter().map(|(p, _, _)| *p).collect();
    let normals: Vec<_> = vertices.iter().map(|(_, n, _)| *n).collect();
    let uvs: Vec<_> = vertices.iter().map(|(_, _, uv)| *uv).collect();

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(indices));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh
}
