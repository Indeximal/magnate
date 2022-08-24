use bevy::{prelude::*, utils::HashSet};
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
    mouse_btn: Res<Input<MouseButton>>,
    indicator: Query<&SelectionIndicator>,
    parents: Query<(&Parent, &GlobalTransform)>,
    children: Query<&Children>,
    mut triangles: Query<(&mut Transform, &mut TriangleTile)>,
) {
    if !mouse_btn.any_just_pressed([MouseButton::Left, MouseButton::Right]) {
        return;
    }

    let indicator = indicator
        .get_single()
        .expect("Indicator hasn't been spawned yet!");

    let selected_triggers: Vec<_> = indicator
        .selected_triggers
        .iter()
        .filter_map(|eid| parents.get(*eid).ok())
        .collect();

    if selected_triggers.is_empty() {
        // Nothing selected, so do nothing
        return;
    }

    // This way I don't have to update another coordinate in the triangle vertices.
    let anchor: VertexCoord = FromWorldPosition::from_world_pos(
        selected_triggers
            .first()
            .expect("vector is not empty")
            .1
            .translation()
            .truncate(),
    );

    // Entity id of all triangles that are either parent of a selector or siblings of a parent of a selector
    // All triangles must have a parent for this to work
    let triangles_to_be_rotated: HashSet<Entity> = selected_triggers
        .iter()
        .filter_map(|(selector_par, _)| parents.get(selector_par.get()).ok())
        .filter_map(|(triangle_par, _)| children.get(triangle_par.get()).ok())
        .flat_map(|clump_children| clump_children.iter())
        .cloned()
        .collect();

    for eid in triangles_to_be_rotated {
        if let Ok((mut transf, mut coord)) = triangles.get_mut(eid) {
            let new_vertex: FaceCoord = if mouse_btn.just_pressed(MouseButton::Left) {
                // Counter clockwise
                coord.position.rotate_counter_clockwise(anchor)
            } else if mouse_btn.just_pressed(MouseButton::Right) {
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
