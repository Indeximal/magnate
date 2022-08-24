use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    sprite::MaterialMesh2dBundle,
};
use bevy_point_selection::Selectable;

/// Coordate of the verticies of the triangle grid. X is viewport towards right and Y is towards upper right.
pub use bevy::prelude::IVec2 as VertexCoord;
/// Describes the left vertex and whether the triangle points up or down.
pub type FaceCoord = (VertexCoord, TriangleOrientation);

pub const SQRT3_HALF: f32 = 0.866025404;

const TRIANGLE_SIDE: f32 = 1.0;
const TRIANGLE_Z: f32 = 100.;
const SELECTABLE_RADIUS: f32 = 0.25 * TRIANGLE_SIDE;

const X_DIR: Vec2 = Vec2::new(TRIANGLE_SIDE, 0.);
const Y_DIR: Vec2 = Vec2::new(0.5 * TRIANGLE_SIDE, SQRT3_HALF * TRIANGLE_SIDE);
const W_DIR: Vec2 = Vec2::new(-0.5 * TRIANGLE_SIDE, SQRT3_HALF * TRIANGLE_SIDE);
const ISO_TO_ORTHO: Mat2 = Mat2::from_cols(X_DIR, Y_DIR);

// there is no IMat :(
const ISO_LEFT_ROT: Mat2 = Mat2::from_cols(Vec2::new(1., -1.), Vec2::new(1., 0.));

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TriangleOrientation {
    PointingUp,
    PointingDown,
}

/// Required for the Component derive of [`TriangleTile`]
impl Default for TriangleOrientation {
    fn default() -> Self {
        TriangleOrientation::PointingUp
    }
}

#[derive(Component, Default, Debug, Clone)]
pub struct TriangleTile {
    pub position: FaceCoord,
}

trait PositionInWorld {
    fn to_world_pos(&self, z: f32) -> Transform;
}

impl PositionInWorld for VertexCoord {
    fn to_world_pos(&self, z: f32) -> Transform {
        let xy = ISO_TO_ORTHO * self.as_vec2();
        Transform {
            translation: xy.extend(z),
            ..Default::default()
        }
    }
}

impl PositionInWorld for FaceCoord {
    fn to_world_pos(&self, z: f32) -> Transform {
        self.0.to_world_pos(z)
    }
}

trait RotateAroundVertex {
    fn rotate_clockwise(&self, anchor: VertexCoord) -> Self;
    fn rotate_counter_clockwise(&self, anchor: VertexCoord) -> Self;
}

impl RotateAroundVertex for FaceCoord {
    fn rotate_clockwise(&self, anchor: VertexCoord) -> Self {
        let d = self.0 - anchor;
        let r = ISO_LEFT_ROT * d.as_vec2();
        let p = anchor + r.as_ivec2();

        match self.1 {
            TriangleOrientation::PointingUp => (p, TriangleOrientation::PointingDown),
            TriangleOrientation::PointingDown => {
                (p - VertexCoord::Y, TriangleOrientation::PointingUp)
            }
        }
    }

    fn rotate_counter_clockwise(&self, anchor: VertexCoord) -> Self {
        let d = self.0 - anchor;
        let r = ISO_LEFT_ROT.inverse() * d.as_vec2();
        let p = anchor + r.as_ivec2();

        match self.1 {
            TriangleOrientation::PointingUp => (
                p + VertexCoord::new(-1, 1),
                TriangleOrientation::PointingDown,
            ),
            TriangleOrientation::PointingDown => (p, TriangleOrientation::PointingUp),
        }
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
            mesh: meshes.add(create_triangle(TRIANGLE_SIDE, coord.1)).into(),
            transform: coord.to_world_pos(TRIANGLE_Z),
            material: materials.add(ColorMaterial::from(Color::NAVY)),
            ..default()
        })
        .insert(TriangleTile { position: coord })
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
fn test_rotation() {
    assert_eq!(
        (VertexCoord::new(0, 0), TriangleOrientation::PointingUp)
            .rotate_clockwise(VertexCoord::ZERO),
        (VertexCoord::new(0, 0), TriangleOrientation::PointingDown)
    );
    assert_eq!(
        (VertexCoord::new(0, 0), TriangleOrientation::PointingUp)
            .rotate_counter_clockwise(VertexCoord::ZERO),
        (VertexCoord::new(-1, 1), TriangleOrientation::PointingDown)
    );
    assert_eq!(
        (VertexCoord::new(0, 0), TriangleOrientation::PointingDown)
            .rotate_clockwise(VertexCoord::X),
        (VertexCoord::new(0, 0), TriangleOrientation::PointingUp)
    );
}
