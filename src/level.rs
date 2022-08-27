use std::time::Duration;

use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    utils::HashSet,
};

use crate::{
    savegame::spawn_level,
    tilemap::{RuneTile, TileCoord, TransformInWorld, TriangleTile, SQRT3_HALF, TRIANGLE_SIDE},
    AssetHandles, GameState, SpriteAssets,
};

pub struct MagnateLevelPlugin;

impl Plugin for MagnateLevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(GameState::Next)
                .with_system(rune_system)
                .with_system(soft_despawn)
                .with_system(scale_animation),
        )
        .add_system_set(
            SystemSet::on_enter(GameState::Next)
                .with_system(initial_load.exclusive_system())
                .with_system(spawn_tutorial),
        )
        .init_resource::<LevelInfo>();
    }
}

#[derive(Component, Default, Debug, Clone)]
pub struct ScaleAnimation {
    pub frequency: f32,
    pub amplitude: f32,
}

#[derive(Component, Default, Debug, Clone)]
pub struct SoftDespawned {
    pub death_time: Duration,
}

#[derive(Component, Default, Debug, Clone)]
pub struct RotationHint;

#[derive(Component, Default, Debug, Clone)]
pub struct ReloadHint;

pub struct LevelInfo {
    pub current: usize,
    pub win_animation_progress: Option<f32>,
    pub should_reload: bool,
}

impl Default for LevelInfo {
    fn default() -> Self {
        Self {
            current: 1,
            win_animation_progress: None,
            should_reload: false,
        }
    }
}

fn rune_system(
    mut runes: Query<(&RuneTile, &mut TextureAtlasSprite, &mut Transform)>,
    added_runes: Query<Entity, Added<RuneTile>>,
    changed_triangles: Query<Entity, Changed<TriangleTile>>,
    all_triangles: Query<&TriangleTile>,
    mut level: ResMut<LevelInfo>,
    time: Res<Time>,
) {
    if let Some(progress) = level.win_animation_progress {
        if progress >= 0.6 {
            level.current += 1;
            level.should_reload = true;
            level.win_animation_progress = None;
        } else {
            for (_, _, mut transf) in runes.iter_mut() {
                transf.scale *= 1. + progress;
            }
            level.win_animation_progress = Some(progress + time.delta_seconds());
        }
        return;
    }

    if changed_triangles.is_empty() && added_runes.is_empty() {
        return;
    }
    let all_triangles: HashSet<TileCoord> = all_triangles.iter().map(|tri| tri.position).collect();

    let mut total_runes = 0;
    let mut fulfilled_runes = 0;
    for (rune, mut sprite, _) in runes.iter_mut() {
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
        level.win_animation_progress = Some(0.);
    }
}

fn soft_despawn(
    mut commands: Commands,
    mut affected: Query<(Entity, &mut Transform, &SoftDespawned)>,
    time: Res<Time>,
) {
    let death_span = 1.;
    for (id, mut transf, anim) in affected.iter_mut() {
        let diff_time = (time.time_since_startup() - anim.death_time).as_secs_f32();
        if diff_time < death_span {
            let scale_factor = 1. - diff_time / death_span;
            transf.scale *= Vec3::splat(scale_factor);
        } else {
            commands.entity(id).despawn_recursive();
        }
    }
}

fn scale_animation(mut affected: Query<(&mut Transform, &ScaleAnimation)>, time: Res<Time>) {
    for (mut transf, anim) in affected.iter_mut() {
        let scale = 1.
            + f32::sin(
                time.time_since_startup().as_secs_f32()
                    * anim.frequency
                    * 2.
                    * std::f32::consts::PI,
            ) * anim.amplitude;
        transf.scale = Vec3::splat(scale);
    }
}

fn spawn_tutorial(mut commands: Commands, sprites: Res<SpriteAssets>) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: sprites.reload_hint.clone(),
            transform: Transform {
                translation: Vec3::new(400., 300., 900.),
                scale: Vec3::splat(0.5),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Name::new("Reload Hint"))
        .insert(ReloadHint);

    commands
        .spawn_bundle(SpriteBundle {
            texture: sprites.rotate_hint.clone(),
            sprite: Sprite {
                custom_size: Some(Vec2::splat(0.4 * TRIANGLE_SIDE)),
                color: Color::rgba_u8(199, 172, 252, 230),
                ..Default::default()
            },
            transform: {
                let mut transf = crate::tilemap::VertexCoord::new(0, 1).to_world_pos();
                transf.translation.z = 800.;
                transf
            },
            ..Default::default()
        })
        .insert(ScaleAnimation {
            frequency: 0.2,
            amplitude: 0.13,
        })
        .insert(Name::new("Rotation Hint"))
        .insert(RotationHint);
}

/// Spawn the first level
fn initial_load(world: &mut World) {
    // maybe use asset loader lib?
    //  see https://github.com/NiklasEi/bevy_asset_loader/blob/main/bevy_asset_loader/examples/custom_dynamic_assets.rs
    let ruby_sprite = world.resource::<SpriteAssets>().ruby_triangle.clone();
    let grey_sprite = world.resource::<SpriteAssets>().grey_triangle.clone();
    let meshes = world
        .resource_mut::<Assets<Mesh>>()
        .add(create_triangle_mesh(TRIANGLE_SIDE));
    let ruby_material = world
        .resource_mut::<Assets<ColorMaterial>>()
        .add(ColorMaterial {
            color: Color::WHITE,
            texture: Some(ruby_sprite),
        });
    let grey_material = world
        .resource_mut::<Assets<ColorMaterial>>()
        .add(ColorMaterial {
            color: Color::WHITE,
            texture: Some(grey_sprite),
        });
    let assets = AssetHandles {
        triangle_mesh: meshes,
        triangle_material: ruby_material,
        immovable_material: grey_material,
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
