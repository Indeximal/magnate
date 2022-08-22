//! Inspired by https://github.com/Anshorei/bevy_rei/tree/master/bevy_interact_2d

use bevy::prelude::*;

pub struct PointSelectionPlugin;

impl Plugin for PointSelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(selection_system);
    }
}

/// Add this component to the Camera
#[derive(Component)]
pub struct SelectionSource;

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

fn selection_system(
    mut cursor_events: EventReader<CursorMoved>,
    windows: Res<Windows>,
    sources: Query<(&Camera, &GlobalTransform), With<SelectionSource>>,
    mut sinks: Query<(Entity, &mut Selectable, &GlobalTransform)>,
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

        // The intention behind those vectors is to only dereference the `Selectable` if anything changes,
        // so that the Changed<> filter can be used.
        let mut selected: Vec<Entity> = Vec::new();
        let mut deselected: Vec<Entity> = Vec::new();
        for (id, selectable, transform) in sinks.iter() {
            let dist = transform
                .translation()
                .truncate()
                .distance_squared(cursor_position);
            let radius_sq = selectable.selection_radius * selectable.selection_radius;
            if dist <= radius_sq && !selectable.is_selected {
                selected.push(id);
            }
            if dist > radius_sq && selectable.is_selected {
                deselected.push(id);
            }
        }

        for id in selected {
            if let Ok(mut s) = sinks.get_component_mut::<Selectable>(id) {
                info!("selected");
                s.is_selected = true;
            }
        }
        for id in deselected {
            if let Ok(mut s) = sinks.get_component_mut::<Selectable>(id) {
                info!("deselected");
                s.is_selected = false;
            }
        }
    };
}
