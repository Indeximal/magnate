use bevy::{prelude::*, utils::HashSet};
use bevy_point_selection::SelectionIndicator;

use crate::{
    tilemap::{
        FaceCoord, FromWorldPosition, PositionInWorld, RotateAroundVertex, TriangleTile,
        VertexCoord,
    },
    GameState, SpriteAssets,
};

#[derive(Component, Default)]
struct SelectedTrianglesState {
    /// The entity ids of all currently selected [`TriangleTile`]
    pub selected_set: HashSet<Entity>,
    /// The coordinate of the rotation point if it exists, otherwise undefined
    pub anchor: VertexCoord,
}

pub struct TriangleRotationPlugin;

impl Plugin for TriangleRotationPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::Next).with_system(spawn_selector))
            .add_system_set(
                SystemSet::on_update(GameState::Next)
                    .with_system(triangle_selection_system)
                    .with_system(rotation_system.after(triangle_selection_system)),
            );
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
        .insert(SelectedTrianglesState::default())
        .insert(Name::new("Triangle Selector"));
}

/// This system walkes the hierarchy if the vertex selection changed to pre calculate all the
/// affected triangles.
fn triangle_selection_system(
    mut indicator: Query<
        (&mut SelectedTrianglesState, &SelectionIndicator),
        Changed<SelectionIndicator>,
    >,
    parents: Query<(&Parent, &GlobalTransform)>,
    children: Query<&Children>,
) {
    let (mut selection_state, indicator) = match indicator.get_single_mut() {
        Ok(x) => x,
        Err(_) => return, // only update when the selection changed
    };

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

    selection_state.anchor = anchor;
    selection_state.selected_set = triangles_to_be_rotated;
}

/// This system rotates selected triangles on mouse click
fn rotation_system(
    mouse_btn: Res<Input<MouseButton>>,
    selection: Query<&SelectedTrianglesState>,
    mut triangles: Query<(&mut Transform, &mut TriangleTile)>,
) {
    if !mouse_btn.any_just_pressed([MouseButton::Left, MouseButton::Right]) {
        return;
    }

    let selection = selection
        .get_single()
        .expect("Indicator hasn't been spawned yet!");

    for eid in selection.selected_set.iter() {
        if let Ok((mut transf, mut coord)) = triangles.get_mut(*eid) {
            let new_vertex: FaceCoord = if mouse_btn.just_pressed(MouseButton::Left) {
                // Counter clockwise
                coord.position.rotate_counter_clockwise(selection.anchor)
            } else if mouse_btn.just_pressed(MouseButton::Right) {
                // Clockwise
                coord.position.rotate_clockwise(selection.anchor)
            } else {
                // Do nothing
                coord.position
            };
            coord.position = new_vertex;
            *transf = new_vertex.to_world_pos(transf.translation.z);
        }
    }
}
