use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    sprite::MaterialMesh2dBundle,
};

pub use bevy::prelude::IVec3 as VertexCoord;
use bevy_point_selection::Selectable;
/// Describes the left vertex and whether the triangle points up or down
pub type FaceCoord = (VertexCoord, TriangleOrientation);

pub const SQRT3_HALF: f32 = 0.866025404;

const X_DIR: Vec2 = Vec2::X;
const Y_DIR: Vec2 = Vec2::new(0.5, SQRT3_HALF);
const W_DIR: Vec2 = Vec2::new(-0.5, SQRT3_HALF);

const SELECTABLE_RADIUS: f32 = 0.4;

#[derive(Debug, Clone, Copy)]
pub enum TriangleOrientation {
    PointingUp,
    PointingDown,
}

// /// inspired by https://docs.rs/bevy_ecs_tilemap/latest/bevy_ecs_tilemap/tiles/struct.TileStorage.html
// #[derive(Component, Default, Debug, Clone)]
// pub struct TileStorage {
//     tiles: Vec<(Option<Entity>, Option<Entity>)>,
//     size: (u32, u32),
// }

trait PositionInWorld {
    fn to_world_pos(&self) -> Transform;
}

impl PositionInWorld for VertexCoord {
    fn to_world_pos(&self) -> Transform {
        let xy = self.x as f32 * X_DIR + self.y as f32 * Y_DIR + self.z as f32 * W_DIR;
        Transform {
            translation: xy.extend(100.),
            ..Default::default()
        }
    }
}

impl PositionInWorld for FaceCoord {
    fn to_world_pos(&self) -> Transform {
        self.0.to_world_pos()
    }
}

trait Standarize {
    fn to_standard(&self) -> Self;
}

impl Standarize for VertexCoord {
    fn to_standard(&self) -> Self {
        VertexCoord::new(self.x - self.z, self.y + self.z, 0)
    }
}

/// create a new mesh for a triangle
pub fn create_triangle(size: f32, orientation: TriangleOrientation) -> Mesh {
    // pos  , normal  , uv
    // x y z, nx ny nz, u v
    let vertices = [
        ([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 0.0]),
        ([size, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0]),
        (
            match orientation {
                TriangleOrientation::PointingUp => [size / 2., size * SQRT3_HALF, 0.0],
                TriangleOrientation::PointingDown => [size / 2., -size * SQRT3_HALF, 0.0],
            },
            [0.0, 0.0, 1.0],
            [1.0, 0.5],
        ),
    ];
    let indices = match orientation {
        TriangleOrientation::PointingUp => Indices::U32(vec![0, 1, 2]),
        TriangleOrientation::PointingDown => Indices::U32(vec![0, 2, 1]),
    };

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

pub fn spawn_triangle(
    commands: &mut Commands,
    coord: FaceCoord,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
    // Todo: optimize, don't create a mesh every time?
    // see https://github.com/NiklasEi/bevy_asset_loader/blob/main/bevy_asset_loader/examples/custom_dynamic_assets.rs
    commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: meshes.add(create_triangle(1., coord.1)).into(),
            transform: coord.to_world_pos(),
            material: materials.add(ColorMaterial::from(Color::NAVY)),
            ..default()
        })
        .with_children(|builder| {
            builder
                .spawn_bundle(TransformBundle::from_transform(Transform::default()))
                .insert(Selectable::new(SELECTABLE_RADIUS));
            builder
                .spawn_bundle(TransformBundle::from_transform(
                    Transform::from_translation(X_DIR.extend(0.)),
                ))
                .insert(Selectable::new(SELECTABLE_RADIUS));
            builder
                .spawn_bundle(TransformBundle::from_transform(match coord.1 {
                    TriangleOrientation::PointingUp => {
                        Transform::from_translation(Y_DIR.extend(0.))
                    }
                    TriangleOrientation::PointingDown => {
                        Transform::from_translation(-W_DIR.extend(0.))
                    }
                }))
                .insert(Selectable::new(SELECTABLE_RADIUS));
        });
}

#[test]
fn test_standardization() {
    assert_eq!(
        VertexCoord::new(1, 1, 0).to_standard(),
        VertexCoord::new(1, 1, 0)
    );
    assert_eq!(
        VertexCoord::new(1, 0, 1).to_standard(),
        VertexCoord::new(0, 1, 0)
    );
    assert_eq!(
        VertexCoord::new(0, 0, 1).to_standard(),
        VertexCoord::new(-1, 1, 0)
    );
}
