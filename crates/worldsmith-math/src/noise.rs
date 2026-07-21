//! Deterministic procedural noise algorithms.

use crate::numeric::{lerp, smoothstep};

/// Common interface for deterministic 2D noise sources.
pub trait Noise2 {
    /// Returns a repeatable value, usually in `[-1, 1]`.
    fn sample2(&self, x: f64, y: f64) -> f64;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ValueNoise {
    pub seed: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PerlinNoise {
    pub seed: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fbm<N> {
    pub source: N,
    pub octaves: u32,
    pub lacunarity: f64,
    pub gain: f64,
}

impl ValueNoise {
    #[inline]
    pub const fn new(seed: u64) -> Self {
        Self { seed }
    }
}

impl PerlinNoise {
    #[inline]
    pub const fn new(seed: u64) -> Self {
        Self { seed }
    }
}

impl<N> Fbm<N> {
    #[inline]
    pub const fn new(source: N, octaves: u32, lacunarity: f64, gain: f64) -> Self {
        Self {
            source,
            octaves,
            lacunarity,
            gain,
        }
    }
}

impl Noise2 for ValueNoise {
    fn sample2(&self, x: f64, y: f64) -> f64 {
        let x0 = x.floor() as i64;
        let y0 = y.floor() as i64;
        let xf = x - x0 as f64;
        let yf = y - y0 as f64;
        let u = smoothstep(0.0, 1.0, xf);
        let v = smoothstep(0.0, 1.0, yf);
        let a = value_at(self.seed, x0, y0);
        let b = value_at(self.seed, x0 + 1, y0);
        let c = value_at(self.seed, x0, y0 + 1);
        let d = value_at(self.seed, x0 + 1, y0 + 1);
        lerp(lerp(a, b, u), lerp(c, d, u), v)
    }
}

impl Noise2 for PerlinNoise {
    fn sample2(&self, x: f64, y: f64) -> f64 {
        let x0 = x.floor() as i64;
        let y0 = y.floor() as i64;
        let xf = x - x0 as f64;
        let yf = y - y0 as f64;
        let u = fade(xf);
        let v = fade(yf);
        let aa = gradient_dot(self.seed, x0, y0, xf, yf);
        let ba = gradient_dot(self.seed, x0 + 1, y0, xf - 1.0, yf);
        let ab = gradient_dot(self.seed, x0, y0 + 1, xf, yf - 1.0);
        let bb = gradient_dot(self.seed, x0 + 1, y0 + 1, xf - 1.0, yf - 1.0);
        lerp(lerp(aa, ba, u), lerp(ab, bb, u), v).clamp(-1.0, 1.0)
    }
}

impl<N: Noise2> Noise2 for Fbm<N> {
    fn sample2(&self, x: f64, y: f64) -> f64 {
        if self.octaves == 0 {
            return 0.0;
        }
        let mut amplitude = 1.0;
        let mut frequency = 1.0;
        let mut total = 0.0;
        let mut normalization = 0.0;
        for _ in 0..self.octaves {
            total += self.source.sample2(x * frequency, y * frequency) * amplitude;
            normalization += amplitude;
            amplitude *= self.gain;
            frequency *= self.lacunarity;
        }
        total / normalization
    }
}

#[inline]
fn fade(t: f64) -> f64 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn value_at(seed: u64, x: i64, y: i64) -> f64 {
    let h = hash_coords(seed, x, y);
    let unit = (h >> 11) as f64 * (1.0 / ((1u64 << 53) as f64));
    unit * 2.0 - 1.0
}

fn gradient_dot(seed: u64, x: i64, y: i64, dx: f64, dy: f64) -> f64 {
    const GRADIENTS: [(f64, f64); 8] = [
        (1.0, 0.0),
        (-1.0, 0.0),
        (0.0, 1.0),
        (0.0, -1.0),
        (0.707_106_781_186_547_5, 0.707_106_781_186_547_5),
        (-0.707_106_781_186_547_5, 0.707_106_781_186_547_5),
        (0.707_106_781_186_547_5, -0.707_106_781_186_547_5),
        (-0.707_106_781_186_547_5, -0.707_106_781_186_547_5),
    ];
    let g = GRADIENTS[(hash_coords(seed, x, y) as usize) & 7];
    g.0 * dx + g.1 * dy
}

fn hash_coords(seed: u64, x: i64, y: i64) -> u64 {
    let mut h = seed ^ (x as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    h ^= (y as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^= h >> 30;
    h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^= h >> 27;
    h = h.wrapping_mul(0x94D0_49BB_1331_11EB);
    h ^ (h >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_noise_is_deterministic() {
        let a = ValueNoise::new(42).sample2(1.25, -3.5);
        let b = ValueNoise::new(42).sample2(1.25, -3.5);
        assert_eq!(a, b);
    }

    #[test]
    fn perlin_lattice_points_are_zero() {
        assert_eq!(PerlinNoise::new(1).sample2(4.0, 8.0), 0.0);
    }

    #[test]
    fn fbm_zero_octaves_is_zero() {
        assert_eq!(
            Fbm::new(ValueNoise::new(1), 0, 2.0, 0.5).sample2(1.0, 1.0),
            0.0
        );
    }
}
