//! Unit quaternions for stable 3D rotations.

use std::ops::Mul;

use serde::{Deserialize, Serialize};

use crate::{numeric, Vector3};

/// Quaternion stored as `(x, y, z, w)`, with `w` as the scalar component.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Quaternion {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

impl Quaternion {
    pub const IDENTITY: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 1.0,
    };

    #[inline]
    pub const fn new(x: f64, y: f64, z: f64, w: f64) -> Self {
        Self { x, y, z, w }
    }

    pub fn from_axis_angle(axis: Vector3, angle_radians: f64) -> Self {
        let normal = axis.normalize();
        if normal == Vector3::ZERO {
            return Self::IDENTITY;
        }
        let half = angle_radians * 0.5;
        let (s, c) = half.sin_cos();
        Self::new(normal.x * s, normal.y * s, normal.z * s, c).normalize()
    }

    pub fn from_euler_xyz(x_radians: f64, y_radians: f64, z_radians: f64) -> Self {
        let qx = Self::from_axis_angle(Vector3::X, x_radians);
        let qy = Self::from_axis_angle(Vector3::Y, y_radians);
        let qz = Self::from_axis_angle(Vector3::Z, z_radians);
        qz * qy * qx
    }

    #[inline]
    pub fn magnitude(self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt()
    }

    #[inline]
    pub fn normalize(self) -> Self {
        let len = self.magnitude();
        if len <= f64::EPSILON {
            Self::IDENTITY
        } else {
            Self::new(self.x / len, self.y / len, self.z / len, self.w / len)
        }
    }

    #[inline]
    pub fn conjugate(self) -> Self {
        Self::new(-self.x, -self.y, -self.z, self.w)
    }

    #[inline]
    pub fn rotate_vector(self, v: Vector3) -> Vector3 {
        let qv = Vector3::new(self.x, self.y, self.z);
        let t = 2.0 * qv.cross(v);
        v + self.w * t + qv.cross(t)
    }

    pub fn lerp(self, other: Self, t: f64) -> Self {
        Self::new(
            numeric::lerp(self.x, other.x, t),
            numeric::lerp(self.y, other.y, t),
            numeric::lerp(self.z, other.z, t),
            numeric::lerp(self.w, other.w, t),
        )
        .normalize()
    }

    pub fn slerp(self, mut other: Self, t: f64) -> Self {
        let mut dot = self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w;
        if dot < 0.0 {
            other = Self::new(-other.x, -other.y, -other.z, -other.w);
            dot = -dot;
        }
        if dot > 0.999_5 {
            return self.lerp(other, t);
        }
        let theta_0 = dot.acos();
        let theta = theta_0 * t;
        let sin_theta = theta.sin();
        let sin_theta_0 = theta_0.sin();
        let s0 = theta.cos() - dot * sin_theta / sin_theta_0;
        let s1 = sin_theta / sin_theta_0;
        Self::new(
            self.x * s0 + other.x * s1,
            self.y * s0 + other.y * s1,
            self.z * s0 + other.z * s1,
            self.w * s0 + other.w * s1,
        )
    }
}

impl Default for Quaternion {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Mul for Quaternion {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self::new(
            self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y,
            self.w * rhs.y - self.x * rhs.z + self.y * rhs.w + self.z * rhs.x,
            self.w * rhs.z + self.x * rhs.y - self.y * rhs.x + self.z * rhs.w,
            self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotates_vector_around_z() {
        let q = Quaternion::from_axis_angle(Vector3::Z, std::f64::consts::FRAC_PI_2);
        let v = q.rotate_vector(Vector3::X);
        assert!(numeric::approx_eq_scaled(v.x, 0.0, 1e-12));
        assert!(numeric::approx_eq_scaled(v.y, 1.0, 1e-12));
    }
}
