//! Two-component Euclidean vector.

use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use serde::{Deserialize, Serialize};

use crate::numeric;

/// A 2D vector with `f64` components.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Vector2 {
    pub x: f64,
    pub y: f64,
}

impl Vector2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub const ONE: Self = Self { x: 1.0, y: 1.0 };
    pub const X: Self = Self { x: 1.0, y: 0.0 };
    pub const Y: Self = Self { x: 0.0, y: 1.0 };

    #[inline]
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y
    }

    #[inline]
    pub fn magnitude(self) -> f64 {
        self.dot(self).sqrt()
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
        )
    }

    /// Component-wise multiplication.
    #[inline]
    pub fn hadamard(self, other: Self) -> Self {
        Self::new(self.x * other.x, self.y * other.y)
    }

    /// Projects `self` onto `onto`.
    #[inline]
    pub fn project(self, onto: Self) -> Self {
        let denom = onto.magnitude_squared();
        if denom <= f64::EPSILON {
            Self::ZERO
        } else {
            onto * (self.dot(onto) / denom)
        }
    }

    /// Rejection of `self` from `from` (component perpendicular to `from`).
    #[inline]
    pub fn reject(self, from: Self) -> Self {
        self - self.project(from)
    }

    /// 2D scalar cross product (signed parallelogram area).
    #[inline]
    pub fn cross(self, other: Self) -> f64 {
        self.x * other.y - self.y * other.x
    }

    /// Rotates this vector by `angle_radians` counter-clockwise.
    #[inline]
    pub fn rotate(self, angle_radians: f64) -> Self {
        let (s, c) = angle_radians.sin_cos();
        Self::new(self.x * c - self.y * s, self.x * s + self.y * c)
    }
}

impl Add for Vector2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for Vector2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Mul<f64> for Vector2 {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl Mul<Vector2> for f64 {
    type Output = Vector2;
    fn mul(self, rhs: Vector2) -> Vector2 {
        rhs * self
    }
}

impl Div<f64> for Vector2 {
    type Output = Self;
    fn div(self, rhs: f64) -> Self {
        Self::new(self.x / rhs, self.y / rhs)
    }
}

impl Neg for Vector2 {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y)
    }
}

impl AddAssign for Vector2 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl SubAssign for Vector2 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl MulAssign<f64> for Vector2 {
    fn mul_assign(&mut self, rhs: f64) {
        *self = *self * rhs;
    }
}

impl DivAssign<f64> for Vector2 {
    fn div_assign(&mut self, rhs: f64) {
        *self = *self / rhs;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_unit_length() {
        let v = Vector2::new(3.0, 4.0).normalize();
        assert!((v.magnitude() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn rotate_90_degrees() {
        let v = Vector2::X.rotate(std::f64::consts::FRAC_PI_2);
        assert!((v.x).abs() < 1e-12);
        assert!((v.y - 1.0).abs() < 1e-12);
    }
}
