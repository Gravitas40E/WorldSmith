//! Three-component Euclidean vector.

use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use serde::{Deserialize, Serialize};

use crate::numeric;

/// A 3D vector with `f64` components.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3 {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    pub const ONE: Self = Self {
        x: 1.0,
        y: 1.0,
        z: 1.0,
    };
    pub const X: Self = Self {
        x: 1.0,
        y: 0.0,
        z: 0.0,
    };
    pub const Y: Self = Self {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    };
    pub const Z: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 1.0,
    };

    #[inline]
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    #[inline]
    pub fn cross(self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    #[inline]
    pub fn magnitude(self) -> f64 {
        self.magnitude_squared().sqrt()
    }

    #[inline]
    pub fn magnitude_squared(self) -> f64 {
        self.dot(self)
    }

    #[inline]
    pub fn distance(self, other: Self) -> f64 {
        (self - other).magnitude()
    }

    #[inline]
    pub fn normalize(self) -> Self {
        let len = self.magnitude();
        if len <= f64::EPSILON {
            Self::ZERO
        } else {
            self / len
        }
    }

    #[inline]
    pub fn lerp(self, other: Self, t: f64) -> Self {
        Self::new(
            numeric::lerp(self.x, other.x, t),
            numeric::lerp(self.y, other.y, t),
            numeric::lerp(self.z, other.z, t),
        )
    }

    #[inline]
    pub fn hadamard(self, other: Self) -> Self {
        Self::new(self.x * other.x, self.y * other.y, self.z * other.z)
    }

    #[inline]
    pub fn project(self, onto: Self) -> Self {
        let denom = onto.magnitude_squared();
        if denom <= f64::EPSILON {
            Self::ZERO
        } else {
            onto * (self.dot(onto) / denom)
        }
    }

    #[inline]
    pub fn reject(self, from: Self) -> Self {
        self - self.project(from)
    }
}

impl Add for Vector3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub for Vector3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Mul<f64> for Vector3 {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl Mul<Vector3> for f64 {
    type Output = Vector3;
    fn mul(self, rhs: Vector3) -> Vector3 {
        rhs * self
    }
}

impl Div<f64> for Vector3 {
    type Output = Self;
    fn div(self, rhs: f64) -> Self {
        Self::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

impl Neg for Vector3 {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
    }
}

impl AddAssign for Vector3 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl SubAssign for Vector3 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl MulAssign<f64> for Vector3 {
    fn mul_assign(&mut self, rhs: f64) {
        *self = *self * rhs;
    }
}

impl DivAssign<f64> for Vector3 {
    fn div_assign(&mut self, rhs: f64) {
        *self = *self / rhs;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cross_product_axes() {
        assert_eq!(Vector3::X.cross(Vector3::Y), Vector3::Z);
    }

    #[test]
    fn projection_onto_axis() {
        let v = Vector3::new(3.0, 4.0, 5.0).project(Vector3::Y);
        assert_eq!(v, Vector3::new(0.0, 4.0, 0.0));
    }
}
