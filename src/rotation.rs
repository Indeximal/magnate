use bevy::prelude::*;
use bevy_point_selection::SelectionIndicator;

use crate::{tilemap::TriangleTile, GameState, SpriteAssets};

pub struct TriangleRotationPlugin;

impl Plugin for TriangleRotationPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::Next).with_system(spawn_selector))
            .add_system_set(SystemSet::on_update(GameState::Next).with_system(rotation_system));
    }
}

/// This system must be run on startup after assets where loaded to spawn the [`SelectionIndicator`].
/// It holds both the sprite for user feedback and a vector of the selected triangles.
fn spawn_selector(mut commands: Commands, assets: Res<SpriteAssets>) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: assets.circle.clone(),
            sprite: Sprite {
                custom_size: Some(Vec2::splat(0.4)),
                color: Color::rgb_u8(87, 207, 255),
                ..Default::default()
            },
            visibility: Visibility { is_visible: false },
            ..Default::default()
        })
        .insert(SelectionIndicator::new())
        .insert(Name::new("Triangle Selector"));
}

fn rotation_system(
    triggers: Query<&Parent>,
    triangles: Query<&TriangleTile>,
    indicator: Query<&SelectionIndicator>,
) {
    let indicator = indicator
        .get_single()
        .expect("Indicator hasn't been spawned yet!");

    let selected_triangles_iter = indicator
        .selected_triggers
        .iter()
        .flat_map(|eid| triggers.get(*eid))
        .flat_map(|par| triangles.get(par.get()));

    for tri in selected_triangles_iter {
        info!("{:?}", tri.position);
    }
}
