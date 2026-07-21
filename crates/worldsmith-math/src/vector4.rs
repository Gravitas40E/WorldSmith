//! Four-component vector for homogeneous coordinates and packed values.

use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use serde::{Deserialize, Serialize};

use crate::{numeric, Vector3};

/// A 4D vector with `f64` components.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Vector4 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

impl Vector4 {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 0.0,
    };
    pub const ONE: Self = Self {
        x: 1.0,
        y: 1.0,
        z: 1.0,
        w: 1.0,
    };

    #[inline]
    pub const fn new(x: f64, y: f64, z: f64, w: f64) -> Self {
        Self { x, y, z, w }
    }

    #[inline]
    pub const fn from_vector3(v: Vector3, w: f64) -> Self {
        Self::new(v.x, v.y, v.z, w)
    }

    #[inline]
    pub const fn xyz(self) -> Vector3 {
        Vector3::new(self.x, self.y, self.z)
    }

    #[inline]
    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
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
            numeric::lerp(self.w, other.w, t),
        )
    }
}

impl Add for Vector4 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(
            self.x + rhs.x,
            self.y + rhs.y,
            self.z + rhs.z,
            self.w + rhs.w,
        )
    }
}

impl Sub for Vector4 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(
            self.x - rhs.x,
            self.y - rhs.y,
            self.z - rhs.z,
            self.w - rhs.w,
        )
    }
}

impl Mul<f64> for Vector4 {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs, self.w * rhs)
    }
}

impl Mul<Vector4> for f64 {
    type Output = Vector4;
    fn mul(self, rhs: Vector4) -> Vector4 {
        rhs * self
    }
}

impl Div<f64> for Vector4 {
    type Output = Self;
    fn div(self, rhs: f64) -> Self {
        Self::new(self.x / rhs, self.y / rhs, self.z / rhs, self.w / rhs)
    }
}

impl Neg for Vector4 {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y, -self.z, -self.w)
    }
}

impl AddAssign for Vector4 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl SubAssign for Vector4 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl MulAssign<f64> for Vector4 {
    fn mul_assign(&mut self, rhs: f64) {
        *self = *self * rhs;
    }
}

impl DivAssign<f64> for Vector4 {
    fn div_assign(&mut self, rhs: f64) {
        *self = *self / rhs;
    }
}
