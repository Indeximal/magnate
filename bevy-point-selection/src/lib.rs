//! Inspired by https://github.com/Anshorei/bevy_rei/tree/master/bevy_interact_2d

use bevy::prelude::*;

pub struct PointSelectionPlugin;

impl Plugin for PointSelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(selection_system).add_system(update_selector);
    }
}

/// Add this component to the Camera
#[derive(Component)]
pub struct SelectionSource;

/// Use with a `Changed<Selectable>` filter to skip unchanged Selectables.
/// Somewhat analogous to bevy_ui Interactible
///
/// Entities must have a [`GlobalTransform`] components for the system to update `is_selected`.
///
/// todo: add other colliders, custom offset?
#[derive(Component)]
pub struct Selectable {
    /// Radius from center of transform in world units
    pub selection_radius: f32,
    pub is_selected: bool,
}

impl Selectable {
    pub fn new(radius: f32) -> Selectable {
        Selectable {
            selection_radius: radius,
            is_selected: false,
        }
    }
}

/// Entities with this component will be moved to a selected [`Selectable`] or be set to invisible
/// if none are selected. Entities must have a [`Transform`] and [`Visibility`] components for this to
/// take effect.
#[derive(Component)]
pub struct SelectionIndicator {
    /// The entity ids of the currently selected [`Selectable`]
    pub selected_triggers: Vec<Entity>,
}

impl SelectionIndicator {
    pub fn new() -> SelectionIndicator {
        SelectionIndicator {
            selected_triggers: Vec::new(),
        }
    }
}

/// This system updates Selectable components based on the cursor position
fn selection_system(
    mut cursor_events: EventReader<CursorMoved>,
    windows: Res<Windows>,
    sources: Query<(&Camera, &GlobalTransform), With<SelectionSource>>,
    mut sinks: Query<(&mut Selectable, &GlobalTransform)>,
) {
    let (window_id, cursor_pos) = match cursor_events.iter().last() {
        Some(evt) => (evt.id, evt.position),
        None => return,
    };

    let window = match windows.get(window_id) {
        Some(window) => window,
        None => return,
    };

    // todo: skip if camera is not displayed on the window?
    // See bevy_ui `ui_focus_system`
    if let Ok((camera, global_transform)) = sources.get_single() {
        let projection_matrix = camera.projection_matrix();
        let screen_size = Vec2::from([window.width() as f32, window.height() as f32]);
        let cursor_position_ndc = (cursor_pos / screen_size) * 2.0 - Vec2::from([1.0, 1.0]);
        let camera_matrix = global_transform.compute_matrix();
        let ndc_to_world = camera_matrix * projection_matrix.inverse();
        let cursor_position = ndc_to_world
            .transform_point3(cursor_position_ndc.extend(1.0))
            .truncate();

        // Calculationg the distance and checking for overlap does not trigger change detection
        for (mut selectable, transform) in sinks.iter_mut() {
            let dist = transform
                .translation()
                .truncate()
                .distance_squared(cursor_position);
            let radius_sq = selectable.selection_radius * selectable.selection_radius;
            if dist <= radius_sq && !selectable.is_selected {
                // this triggers change detection
                selectable.as_mut().is_selected = true;
            }
            if dist > radius_sq && selectable.is_selected {
                // this triggers change detection
                selectable.as_mut().is_selected = false;
            }
        }
    };
}

/// This system updates the vector of selected [`Selectable`]. It also sets the visibility of the indicator
/// and if applicable its position as well. If multiple [`Selectable`] are selected the position is chooses
/// arbitrary.
fn update_selector(
    mut indicator: Query<(&mut Visibility, &mut Transform, &mut SelectionIndicator)>,
    triggers: Query<(Entity, &GlobalTransform, &Selectable), Changed<Selectable>>,
) {
    // Early return if there is no indicator or it hasn't been spawned
    let (mut visi, mut transf, mut indic) = match indicator.get_single_mut() {
        Ok(x) => x,
        Err(_) => return,
    };

    // Early return if nothing changed, then the below Vector is empty iff
    // all changes where because triggers were deselected.
    if triggers.is_empty() {
        return;
    }

    let all_selected: Vec<_> = triggers
        .iter()
        .filter(|(_, _, sel)| sel.is_selected)
        .collect();

    visi.is_visible = !all_selected.is_empty();
    indic.selected_triggers = all_selected.iter().map(|(e, _, _)| e).cloned().collect();
    transf.translation = all_selected
        .first()
        .map(|(_, t, _)| t.translation())
        .unwrap_or_default();
}
