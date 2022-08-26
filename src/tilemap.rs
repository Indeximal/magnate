use bevy::prelude::*;

/// Coordate of the verticies of the triangle grid. X is viewport towards right and Y is towards upper right.
pub use bevy::prelude::IVec2 as VertexCoord;
use serde::{Deserialize, Serialize};
/// Describes the left vertex and whether the triangle points up or down.
/// If it is pointing down, the mesh is rotated a sixth turn clockwise.
pub type TileCoord = (VertexCoord, TriangleOrient);

pub const TRIANGLE_SIDE: f32 = 85.0;
pub const SQRT3_HALF: f32 = 0.866025404;
pub const X_DIR: Vec2 = Vec2::new(TRIANGLE_SIDE, 0.);
pub const Y_DIR: Vec2 = Vec2::new(0.5 * TRIANGLE_SIDE, SQRT3_HALF * TRIANGLE_SIDE);
const ISO_TO_ORTHO: Mat2 = Mat2::from_cols(X_DIR, Y_DIR);

const ZERO_OFFSET: Vec2 = Vec2::new(11., -34.);
const TRIANGLE_Z: f32 = 500.;
const RUNE_Z: f32 = 600.;

// there is no IMat :(
const ISO_LEFT_ROT: Mat2 = Mat2::from_cols(Vec2::new(1., -1.), Vec2::new(1., 0.));

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TriangleOrient {
    #[default]
    PointingUp,
    PointingDown,
}

#[derive(Component, Default, Debug, Clone, Serialize, Deserialize)]
pub struct RuneTile {
    pub position: TileCoord,
}

#[derive(Component, Default, Debug, Clone, Serialize, Deserialize)]
pub struct TriangleTile {
    pub position: TileCoord,
}

pub trait TransformInWorld {
    fn to_world_pos(&self) -> Transform;
}

impl TransformInWorld for VertexCoord {
    fn to_world_pos(&self) -> Transform {
        let xy = ZERO_OFFSET + ISO_TO_ORTHO * self.as_vec2();
        Transform::from_translation(xy.extend(0.))
    }
}

impl TransformInWorld for TriangleTile {
    fn to_world_pos(&self) -> Transform {
        let mut transf = self.position.0.to_world_pos();
        transf.scale = match self.position.1 {
            TriangleOrient::PointingUp => Vec3::ONE,
            TriangleOrient::PointingDown => Vec3::new(1., -1., 1.),
        };
        transf.translation.z = TRIANGLE_Z;

        transf
    }
}

impl TransformInWorld for RuneTile {
    fn to_world_pos(&self) -> Transform {
        let mut transf = self.position.0.to_world_pos();
        transf.translation += match self.position.1 {
            TriangleOrient::PointingUp => (X_DIR + Y_DIR) * 1. / 3.,
            TriangleOrient::PointingDown => (X_DIR - Y_DIR / 2.) * 2. / 3.,
        }
        .extend(0.);
        transf.translation.z = RUNE_Z;
        transf.scale = Vec3::splat(0.35);

        transf
    }
}

pub trait FromWorldPosition {
    fn from_world_pos(pos: Vec2) -> Self;
}

impl FromWorldPosition for VertexCoord {
    fn from_world_pos(pos: Vec2) -> Self {
        let xy = ISO_TO_ORTHO.inverse() * (pos - ZERO_OFFSET);
        xy.round().as_ivec2()
    }
}

impl FromWorldPosition for TileCoord {
    fn from_world_pos(pos: Vec2) -> Self {
        let xy = ISO_TO_ORTHO.inverse() * (pos - ZERO_OFFSET);
        let base = xy.floor();
        let frac = xy - base;

        if frac.x + frac.y <= 1. {
            (base.as_ivec2(), TriangleOrient::PointingUp)
        } else {
            (base.as_ivec2() + IVec2::Y, TriangleOrient::PointingDown)
        }
    }
}

pub trait RotateAroundVertex {
    fn rotated_clockwise(&self, anchor: VertexCoord) -> Self;
    fn rotated_counter_clockwise(&self, anchor: VertexCoord) -> Self;
}

impl RotateAroundVertex for TileCoord {
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

impl IterNeighbors for TileCoord {
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
