//! Inspired by https://github.com/Anshorei/bevy_rei/tree/master/bevy_interact_2d

use bevy::{prelude::*, render::camera::RenderTarget, utils::HashSet};

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
    /// The entity ids of all currently selected [`Selectable`]
    pub selected_triggers: HashSet<Entity>,
}

impl SelectionIndicator {
    pub fn new() -> SelectionIndicator {
        SelectionIndicator {
            selected_triggers: HashSet::new(),
        }
    }
}

pub fn viewport_to_world(
    camera: &Camera,
    cam_transform: &GlobalTransform,
    window: &Window,
) -> Option<Vec2> {
    // Math from https://github.com/Anshorei/bevy_rei/tree/master/bevy_interact_2d
    let cursor_pos = window.cursor_position()?;
    let projection_matrix = camera.projection_matrix();
    let screen_size = Vec2::new(window.width(), window.height());
    let cursor_position_ndc = (cursor_pos / screen_size) * 2.0 - Vec2::from([1.0, 1.0]);
    let camera_matrix = cam_transform.compute_matrix();
    let ndc_to_world = camera_matrix * projection_matrix.inverse();
    let cursor_position = ndc_to_world
        .transform_point3(cursor_position_ndc.extend(1.0))
        .truncate();

    Some(cursor_position)
}

/// This system updates Selectable components based on the cursor position
/// Todo: use ChangeTrackers<GlobalTransform> to update less often, but this doesn't
/// change asymtotic complextity, thus probably is more overhead.
fn selection_system(
    windows: Res<Windows>,
    sources: Query<(&Camera, &GlobalTransform), With<SelectionSource>>,
    mut sinks: Query<(&mut Selectable, &GlobalTransform)>,
) {
    for (camera, cam_transform) in sources.iter() {
        // todo: rewrite with iter functions or let else
        let window = match camera.target {
            RenderTarget::Window(id) => match windows.get(id) {
                Some(window) => window,
                None => continue,
            },
            _ => continue,
        };
        let cursor_position = match viewport_to_world(camera, cam_transform, window) {
            Some(pos) => pos,
            None => continue,
        };

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
    }
}

/// This system updates the set of selected [`Selectable`]. It also sets the visibility of the indicator
/// and if applicable its position as well. If multiple [`Selectable`] are selected, the position is choosen
/// arbitrary.
fn update_selector(
    mut indicator: Query<(&mut Visibility, &mut Transform, &mut SelectionIndicator)>,
    triggers: Query<(Entity, &GlobalTransform, &Selectable), Changed<Selectable>>,
    entities: Query<Entity>,
) {
    // Early return if there is no indicator or it hasn't been spawned yet
    let (mut visi, mut transf, mut indic) = match indicator.get_single_mut() {
        Ok(x) => x,
        Err(_) => return,
    };

    for (eid, trigger_transf, sel) in triggers.iter() {
        if sel.is_selected {
            // Just added
            indic.selected_triggers.insert(eid);
            transf.translation = trigger_transf
                .translation()
                .truncate()
                .extend(transf.translation.z);
        } else {
            // Just removed
            indic.selected_triggers.remove(&eid);
        }
    }

    // Clean up despawned entities
    let orphaned_ids = indic
        .selected_triggers
        .iter()
        .filter(|&&eid| entities.get(eid).is_err())
        .cloned()
        .collect::<Vec<Entity>>();
    for eid in orphaned_ids {
        indic.selected_triggers.remove(&eid);
    }

    // only update when changed
    if visi.is_visible != !indic.selected_triggers.is_empty() {
        visi.is_visible = !indic.selected_triggers.is_empty();
    }
}
