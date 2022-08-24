use std::f32::consts::PI;

use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    sprite::MaterialMesh2dBundle,
};
use bevy_point_selection::Selectable;

/// Coordate of the verticies of the triangle grid. X is viewport towards right and Y is towards upper right.
pub use bevy::prelude::IVec2 as VertexCoord;
/// Describes the left vertex and whether the triangle points up or down.
/// If it is pointing down, the mesh is rotated a sixth turn clockwise.
pub type FaceCoord = (VertexCoord, TriangleOrient);

pub const SQRT3_HALF: f32 = 0.866025404;

const TRIANGLE_SIDE: f32 = 1.0;
const TRIANGLE_Z: f32 = 100.;
const SELECTABLE_RADIUS: f32 = 0.25 * TRIANGLE_SIDE;

const X_DIR: Vec2 = Vec2::new(TRIANGLE_SIDE, 0.);
const Y_DIR: Vec2 = Vec2::new(0.5 * TRIANGLE_SIDE, SQRT3_HALF * TRIANGLE_SIDE);
const ISO_TO_ORTHO: Mat2 = Mat2::from_cols(X_DIR, Y_DIR);

// there is no IMat :(
const ISO_LEFT_ROT: Mat2 = Mat2::from_cols(Vec2::new(1., -1.), Vec2::new(1., 0.));

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriangleOrient {
    PointingUp,
    PointingDown,
}

/// Required for the Component derive of [`TriangleTile`]
impl Default for TriangleOrient {
    fn default() -> Self {
        TriangleOrient::PointingUp
    }
}

#[derive(Component, Default, Debug, Clone)]
pub struct TriangleTile {
    pub position: FaceCoord,
}

pub trait PositionInWorld {
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
        self.0.to_world_pos(z).with_rotation(match self.1 {
            TriangleOrient::PointingUp => Quat::default(),
            TriangleOrient::PointingDown => Quat::from_rotation_z(-PI / 3.),
        })
    }
}

pub trait FromWorldPosition {
    fn from_world_pos(pos: Vec2) -> Self;
}

impl FromWorldPosition for VertexCoord {
    fn from_world_pos(pos: Vec2) -> Self {
        let xy = ISO_TO_ORTHO.inverse() * pos;
        xy.round().as_ivec2()
    }
}

pub trait RotateAroundVertex {
    fn rotated_clockwise(&self, anchor: VertexCoord) -> Self;
    fn rotated_counter_clockwise(&self, anchor: VertexCoord) -> Self;
}

impl RotateAroundVertex for FaceCoord {
    fn rotated_clockwise(&self, anchor: VertexCoord) -> Self {
        let d = self.0 - anchor;
        let r = ISO_LEFT_ROT * d.as_vec2();
        let p = anchor + r.round().as_ivec2();

        match self.1 {
            TriangleOrient::PointingUp => (p, TriangleOrient::PointingDown),
            TriangleOrient::PointingDown => (p - VertexCoord::Y, TriangleOrient::PointingUp),
        }
    }

    fn rotated_counter_clockwise(&self, anchor: VertexCoord) -> Self {
        let d = self.0 - anchor;
        let r = ISO_LEFT_ROT.inverse() * d.as_vec2();
        let p = anchor + r.round().as_ivec2();

        match self.1 {
            TriangleOrient::PointingUp => {
                (p + VertexCoord::new(-1, 1), TriangleOrient::PointingDown)
            }
            TriangleOrient::PointingDown => (p, TriangleOrient::PointingUp),
        }
    }
}

pub trait IterNeighbors {
    type Iter: ExactSizeIterator<Item = Self>;
    fn iter_neighbors(&self) -> Self::Iter;
}

impl IterNeighbors for FaceCoord {
    type Iter = std::array::IntoIter<Self, 3>;
    fn iter_neighbors(&self) -> Self::Iter {
        match self.1 {
            TriangleOrient::PointingUp => [
                (self.0, TriangleOrient::PointingDown),
                (self.0 + IVec2::new(-1, 1), TriangleOrient::PointingDown),
                (self.0 + IVec2::Y, TriangleOrient::PointingDown),
            ],
            TriangleOrient::PointingDown => [
                (self.0, TriangleOrient::PointingUp),
                (self.0 - IVec2::new(-1, 1), TriangleOrient::PointingUp),
                (self.0 - IVec2::Y, TriangleOrient::PointingUp),
            ],
        }
        .into_iter()
    }
}

/// create a new mesh for a triangle
pub fn create_triangle(size: f32) -> Mesh {
    // pos  , normal  , uv
    // x y z, nx ny nz, u v
    let vertices = [
        ([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 0.0]),
        ([size, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0]),
        (
            [size / 2., size * SQRT3_HALF, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.5],
        ),
    ];
    let indices = Indices::U32(vec![0, 1, 2]);

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
) -> Entity {
    // Todo: optimize, don't create a mesh every time?
    // see https://github.com/NiklasEi/bevy_asset_loader/blob/main/bevy_asset_loader/examples/custom_dynamic_assets.rs
    commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: meshes.add(create_triangle(TRIANGLE_SIDE)).into(),
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
                .spawn_bundle(TransformBundle::from_transform(
                    Transform::from_translation(Y_DIR.extend(0.)),
                ))
                .insert(Selectable::new(SELECTABLE_RADIUS));
        })
        .id()
}

#[test]
fn test_rotation() {
    assert_eq!(
        (VertexCoord::new(0, 0), TriangleOrient::PointingUp).rotated_clockwise(VertexCoord::ZERO),
        (VertexCoord::new(0, 0), TriangleOrient::PointingDown)
    );
    assert_eq!(
        (VertexCoord::new(0, 0), TriangleOrient::PointingUp)
            .rotated_counter_clockwise(VertexCoord::ZERO),
        (VertexCoord::new(-1, 1), TriangleOrient::PointingDown)
    );
    assert_eq!(
        (VertexCoord::new(0, 0), TriangleOrient::PointingDown).rotated_clockwise(VertexCoord::X),
        (VertexCoord::new(0, 0), TriangleOrient::PointingUp)
    );
}
