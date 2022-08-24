use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use bevy_point_selection::SelectionIndicator;

use crate::{
    tilemap::{
        FaceCoord, FromWorldPosition, IterNeighbors, PositionInWorld, RotateAroundVertex,
        TriangleTile, VertexCoord,
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
                    .with_system(rotation_system.after(triangle_selection_system))
                    .with_system(merge_system),
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
            transform: Transform::from_xyz(0., 0., 900.),
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
        // Nothing selected, clear the selection
        selection_state.selected_set.clear();
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
    mut triangles: Query<(Entity, &mut Transform, &mut TriangleTile)>,
) {
    if !mouse_btn.any_just_pressed([MouseButton::Left, MouseButton::Right]) {
        return;
    }

    let selection = selection
        .get_single()
        .expect("Indicator hasn't been spawned yet!");

    let mut update_set: Vec<(Entity, FaceCoord)> = Vec::new();
    for (eid, _, coord) in triangles.iter_many(selection.selected_set.iter()) {
        let new_vertex: FaceCoord = if mouse_btn.just_pressed(MouseButton::Left) {
            // Counter clockwise
            coord.position.rotated_counter_clockwise(selection.anchor)
        } else if mouse_btn.just_pressed(MouseButton::Right) {
            // Clockwise
            coord.position.rotated_clockwise(selection.anchor)
        } else {
            // Do nothing
            coord.position
        };
        // delay updating until all collision have been checked
        update_set.push((eid, new_vertex));

        // collision check
        for (other_id, _, other) in triangles.iter() {
            if !selection.selected_set.contains(&other_id) && new_vertex == other.position {
                warn!("Something is in the way!");
                return;
            }
        }
    }

    // Commit updates
    for (eid, new_vertex) in update_set {
        if let Ok((_, mut transf, mut coord)) = triangles.get_mut(eid) {
            coord.position = new_vertex;
            *transf = new_vertex.to_world_pos(transf.translation.z);
        }
    }
}

// This system merges clumps of TriangleTiles that were just moved
fn merge_system(
    mut commands: Commands,
    changed_triangles: Query<(Entity, &TriangleTile), Changed<TriangleTile>>,
    all_triangles: Query<(Entity, &TriangleTile)>,
    parents: Query<&Parent>,
    children: Query<&Children>,
) {
    let all_changed: HashSet<Entity> = changed_triangles.iter().map(|(id, _)| id).collect();
    if all_changed.is_empty() {
        return;
    }

    // Also includes some of the changed triangles
    let all_neighbors: HashMap<FaceCoord, Entity> = changed_triangles
        .iter()
        .flat_map(|(id, p)| p.position.iter_neighbors().zip(std::iter::repeat(id)))
        .collect();

    // Set of all clump pairs that have to be merged. First entry is the just changed one.
    let mut merges: HashSet<(Entity, Entity)> = HashSet::new();

    for (other, tile) in all_triangles.iter() {
        if all_changed.contains(&other) {
            // don't consider any changed triangles
            continue;
        }
        if let Some(&tri) = all_neighbors.get(&tile.position) {
            // tri and other are neighbors now, because tri moved here
            let p1 = parents.get(tri).map(Parent::get);
            let p2 = parents.get(other).map(Parent::get);
            if let (Ok(p1), Ok(p2)) = (p1, p2) {
                merges.insert((p1, p2));
            }
        }
    }

    // Apply merges
    for (p1, p2) in merges {
        if let Ok(new_tiles) = children.get(p2) {
            // fixme: This breaks if two moved clumps try to claim the same tile
            commands
                .entity(p1)
                .push_children(new_tiles.iter().as_slice());
        }
    }
}
