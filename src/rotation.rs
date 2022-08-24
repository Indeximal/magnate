use bevy::prelude::*;
use bevy_point_selection::SelectionIndicator;

use crate::{
    tilemap::{
        FaceCoord, FromWorldPosition, PositionInWorld, RotateAroundVertex, TriangleTile,
        VertexCoord,
    },
    GameState, SpriteAssets,
};

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
    mousebtn: Res<Input<MouseButton>>,
    indicator: Query<&SelectionIndicator>,
    triggers: Query<(&Parent, &GlobalTransform)>,
    mut triangles: Query<(&mut Transform, &mut TriangleTile)>,
) {
    if !mousebtn.any_just_pressed([MouseButton::Left, MouseButton::Right]) {
        return;
    }

    let indicator = indicator
        .get_single()
        .expect("Indicator hasn't been spawned yet!");

    let selected_triggers_iter = indicator
        .selected_triggers
        .iter()
        .flat_map(|eid| triggers.get(*eid));

    for (par, pos) in selected_triggers_iter {
        // This way I don't have to update another coordinate in the triangle vertices.
        let anchor: VertexCoord = FromWorldPosition::from_world_pos(pos.translation().truncate());

        if let Ok((mut transf, mut coord)) = triangles.get_mut(par.get()) {
            let new_vertex: FaceCoord = if mousebtn.just_pressed(MouseButton::Left) {
                // Counter clockwise
                coord.position.rotate_counter_clockwise(anchor)
            } else if mousebtn.just_pressed(MouseButton::Right) {
                // Clockwise
                coord.position.rotate_clockwise(anchor)
            } else {
                // Do nothing
                coord.position
            };
            coord.position = new_vertex;
            *transf = new_vertex.to_world_pos(transf.translation.z);
        }
    }
}
