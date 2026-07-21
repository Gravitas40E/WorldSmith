//! Row-major 4x4 matrices for transforms and projection math.

use std::ops::Mul;

use serde::{Deserialize, Serialize};

use crate::{Quaternion, Vector3, Vector4};

/// Row-major 4x4 matrix.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Matrix4 {
    pub m: [[f64; 4]; 4],
}

impl Matrix4 {
    pub const IDENTITY: Self = Self {
        m: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    };

    #[inline]
    pub const fn new(m: [[f64; 4]; 4]) -> Self {
        Self { m }
    }

    pub fn translation(v: Vector3) -> Self {
        let mut m = Self::IDENTITY;
        m.m[0][3] = v.x;
        m.m[1][3] = v.y;
        m.m[2][3] = v.z;
        m
    }

    pub fn scale(v: Vector3) -> Self {
        Self::new([
            [v.x, 0.0, 0.0, 0.0],
            [0.0, v.y, 0.0, 0.0],
            [0.0, 0.0, v.z, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn rotation(q: Quaternion) -> Self {
        let q = q.normalize();
        let (x2, y2, z2) = (q.x + q.x, q.y + q.y, q.z + q.z);
        let (xx, xy, xz) = (q.x * x2, q.x * y2, q.x * z2);
        let (yy, yz, zz) = (q.y * y2, q.y * z2, q.z * z2);
        let (wx, wy, wz) = (q.w * x2, q.w * y2, q.w * z2);
        Self::new([
            [1.0 - (yy + zz), xy - wz, xz + wy, 0.0],
            [xy + wz, 1.0 - (xx + zz), yz - wx, 0.0],
            [xz - wy, yz + wx, 1.0 - (xx + yy), 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    #[inline]
    pub fn transform_point(self, point: Vector3) -> Vector3 {
        let v = self * Vector4::from_vector3(point, 1.0);
        if v.w.abs() <= f64::EPSILON {
            v.xyz()
        } else {
            v.xyz() / v.w
        }
    }

    #[inline]
    pub fn transform_vector(self, vector: Vector3) -> Vector3 {
        (self * Vector4::from_vector3(vector, 0.0)).xyz()
    }
}

impl Default for Matrix4 {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Mul for Matrix4 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        let mut out = [[0.0; 4]; 4];
        for (row, out_row) in out.iter_mut().enumerate() {
            for (col, out_cell) in out_row.iter_mut().enumerate() {
                *out_cell = self.m[row][0] * rhs.m[0][col]
                    + self.m[row][1] * rhs.m[1][col]
                    + self.m[row][2] * rhs.m[2][col]
                    + self.m[row][3] * rhs.m[3][col];
            }
        }
        Self::new(out)
    }
}

impl Mul<Vector4> for Matrix4 {
    type Output = Vector4;
    fn mul(self, rhs: Vector4) -> Vector4 {
        Vector4::new(
            self.m[0][0] * rhs.x
                + self.m[0][1] * rhs.y
                + self.m[0][2] * rhs.z
                + self.m[0][3] * rhs.w,
            self.m[1][0] * rhs.x
                + self.m[1][1] * rhs.y
                + self.m[1][2] * rhs.z
                + self.m[1][3] * rhs.w,
            self.m[2][0] * rhs.x
                + self.m[2][1] * rhs.y
                + self.m[2][2] * rhs.z
                + self.m[2][3] * rhs.w,
            self.m[3][0] * rhs.x
                + self.m[3][1] * rhs.y
                + self.m[3][2] * rhs.z
                + self.m[3][3] * rhs.w,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translation_moves_points_not_vectors() {
        let m = Matrix4::translation(Vector3::new(1.0, 2.0, 3.0));
        assert_eq!(m.transform_point(Vector3::ONE), Vector3::new(2.0, 3.0, 4.0));
        assert_eq!(m.transform_vector(Vector3::ONE), Vector3::ONE);
    }
}
