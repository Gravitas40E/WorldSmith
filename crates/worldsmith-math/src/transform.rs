//! Position, rotation, and scale composed into a 4x4 transform.

use crate::{Matrix4, Quaternion, Vector3};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    pub translation: Vector3,
    pub rotation: Quaternion,
    pub scale: Vector3,
}

impl Transform {
    pub const IDENTITY: Self = Self {
        translation: Vector3::ZERO,
        rotation: Quaternion::IDENTITY,
        scale: Vector3::ONE,
    };

    #[inline]
    pub const fn new(translation: Vector3, rotation: Quaternion, scale: Vector3) -> Self {
        Self {
            translation,
            rotation,
            scale,
        }
    }

    #[inline]
    pub fn from_translation(translation: Vector3) -> Self {
        Self {
            translation,
            ..Self::IDENTITY
        }
    }

    #[inline]
    pub fn matrix(self) -> Matrix4 {
        Matrix4::translation(self.translation)
            * Matrix4::rotation(self.rotation)
            * Matrix4::scale(self.scale)
    }

    #[inline]
    pub fn transform_point(self, point: Vector3) -> Vector3 {
        self.matrix().transform_point(point)
    }

    #[inline]
    pub fn transform_vector(self, vector: Vector3) -> Vector3 {
        self.matrix().transform_vector(vector)
    }

    #[inline]
    pub fn lerp(self, other: Self, t: f64) -> Self {
        Self::new(
            self.translation.lerp(other.translation, t),
            self.rotation.slerp(other.rotation, t),
            self.scale.lerp(other.scale, t),
        )
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}
