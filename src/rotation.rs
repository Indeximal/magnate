use bevy::prelude::*;
use bevy_point_selection::Selectable;

use crate::{tilemap::TriangleTile, GameState, SpriteAssets};

#[derive(Component)]
pub struct TriangleSelector {
    selected_triggers: Vec<Entity>,
}

pub struct TriangleRotationPlugin;

impl Plugin for TriangleRotationPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::Next).with_system(spawn_selector))
            .add_system_set(
                SystemSet::on_update(GameState::Next)
                    .with_system(update_selector)
                    .with_system(rotation_system),
            );
    }
}

/// This system must be run on startup after assets where loaded to spawn the global selection indicator.
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
        .insert(TriangleSelector {
            selected_triggers: Vec::new(),
        })
        .insert(Name::new("Triangle Selector"));
}

/// This system updates the vector of selected triangles. It also sets the visibility of the indicator
/// and if applicable its position as well. If multiple triangles-triggers are selected the position is choses
/// arbitrarily, but since they should only overlap if they are on the same vertex, this doesn't matter.
fn update_selector(
    triggers: Query<(Entity, &GlobalTransform, &Selectable), Changed<Selectable>>,
    mut indicator: Query<(&mut Visibility, &mut Transform, &mut TriangleSelector)>,
) {
    let (mut visi, mut transf, mut indic) = indicator
        .get_single_mut()
        .expect("Indicator hasn't been spawned yet!");

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

fn rotation_system(
    triggers: Query<&Parent>,
    triangles: Query<&TriangleTile>,
    indicator: Query<&TriangleSelector>,
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
