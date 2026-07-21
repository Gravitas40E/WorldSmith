//! Numerical utility functions used throughout WorldSmith.

use std::f64::consts::PI;

/// Converts radians to degrees.
#[inline]
pub fn to_degrees(radians: f64) -> f64 {
    radians * 180.0 / PI
}

/// Converts degrees to radians.
#[inline]
pub fn to_radians(degrees: f64) -> f64 {
    degrees * PI / 180.0
}

/// Clamps `value` to `[min, max]`.
#[inline]
pub fn clamp(value: f64, min: f64, max: f64) -> f64 {
    value.max(min).min(max)
}

/// Linear interpolation between `a` and `b` by factor `t`.
///
/// `t` is not clamped; values outside `[0, 1]` extrapolate.
#[inline]
pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

/// Computes the interpolation factor that yields `value` between `a` and `b`.
///
/// Returns `0.0` when `a == b` to avoid division by zero.
#[inline]
pub fn inverse_lerp(a: f64, b: f64, value: f64) -> f64 {
    if (b - a).abs() <= f64::EPSILON {
        0.0
    } else {
        (value - a) / (b - a)
    }
}

/// Hermite smooth interpolation: `0` at `edge0`, `1` at `edge1`.
#[inline]
pub fn smoothstep(edge0: f64, edge1: f64, x: f64) -> f64 {
    let t = clamp(inverse_lerp(edge0, edge1, x), 0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Maps `value` from `[in_min, in_max]` to `[out_min, out_max]`.
#[inline]
pub fn remap(value: f64, in_min: f64, in_max: f64, out_min: f64, out_max: f64) -> f64 {
    let t = inverse_lerp(in_min, in_max, value);
    lerp(out_min, out_max, t)
}

/// Returns `true` if `a` and `b` are approximately equal within `epsilon`.
#[inline]
pub fn approx_eq(a: f64, b: f64, epsilon: f64) -> bool {
    (a - b).abs() <= epsilon
}

/// Returns `true` if `a` and `b` are approximately equal using scaled epsilon.
#[inline]
pub fn approx_eq_scaled(a: f64, b: f64, epsilon: f64) -> bool {
    (a - b).abs() <= epsilon * a.abs().max(b.abs()).max(1.0)
}

/// Wraps an angle in radians to `[-π, π)`.
#[inline]
pub fn wrap_angle_radians(angle: f64) -> f64 {
    let two_pi = 2.0 * PI;
    let mut a = angle % two_pi;
    if a >= PI {
        a -= two_pi;
    } else if a < -PI {
        a += two_pi;
    }
    a
}

/// Shortest angular difference from `from` to `to` in radians.
#[inline]
pub fn angle_delta_radians(from: f64, to: f64) -> f64 {
    wrap_angle_radians(to - from)
}

/// Spherical linear interpolation factor helper for angles in radians.
#[inline]
pub fn lerp_angle_radians(a: f64, b: f64, t: f64) -> f64 {
    a + angle_delta_radians(a, b) * t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn degrees_radians_roundtrip() {
        assert!(approx_eq(to_radians(180.0), PI, 1e-12));
        assert!(approx_eq(to_degrees(PI), 180.0, 1e-12));
    }

    #[test]
    fn lerp_endpoints() {
        assert_eq!(lerp(0.0, 10.0, 0.0), 0.0);
        assert_eq!(lerp(0.0, 10.0, 1.0), 10.0);
        assert_eq!(lerp(0.0, 10.0, 0.5), 5.0);
    }

    #[test]
    fn inverse_lerp_roundtrip() {
        let t = inverse_lerp(2.0, 8.0, 5.0);
        assert!(approx_eq(t, 0.5, 1e-12));
    }

    #[test]
    fn smoothstep_edges() {
        assert_eq!(smoothstep(0.0, 1.0, 0.0), 0.0);
        assert_eq!(smoothstep(0.0, 1.0, 1.0), 1.0);
    }

    #[test]
    fn remap_maps_correctly() {
        assert!(approx_eq(remap(5.0, 0.0, 10.0, 100.0, 200.0), 150.0, 1e-12));
    }
}
