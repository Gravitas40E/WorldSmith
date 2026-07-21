//! ChaCha8-backed deterministic random stream.
//!
//! Each stream is seeded independently so modules can derive sub-streams from
//! a master seed without correlated outputs.

use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// Deterministic pseudo-random stream derived from a 64-bit seed.
///
/// Identical seeds produce identical sequences across platforms and runs.
#[derive(Debug, Clone)]
pub struct RngStream {
    inner: ChaCha8Rng,
    seed: u64,
}

impl RngStream {
    /// Creates a new stream from `seed`.
    pub fn new(seed: u64) -> Self {
        Self {
            inner: ChaCha8Rng::seed_from_u64(seed),
            seed,
        }
    }

    /// Returns the seed used to construct this stream.
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Derives an independent sub-stream for a named module or purpose.
    ///
    /// Uses a hash of the parent seed and label so sub-streams are stable and
    /// uncorrelated with the parent sequence order.
    pub fn derive(&self, label: &str) -> Self {
        let mixed = splitmix64(self.seed ^ hash_label(label));
        Self::new(mixed)
    }

    /// Derives a sub-stream indexed by `index` (e.g. tile or body id).
    pub fn derive_indexed(&self, label: &str, index: u64) -> Self {
        let mixed =
            splitmix64(self.seed ^ hash_label(label) ^ index.wrapping_mul(0x9E37_79B9_7F4A_7C15));
        Self::new(mixed)
    }

    /// Uniform `f64` in `[0, 1)`.
    pub fn next_f64(&mut self) -> f64 {
        self.inner.gen::<f64>()
    }

    /// Uniform `f32` in `[0, 1)`.
    pub fn next_f32(&mut self) -> f32 {
        self.inner.gen::<f32>()
    }

    /// Uniform `u32` in `[min, max]` (inclusive).
    pub fn next_u32_inclusive(&mut self, min: u32, max: u32) -> u32 {
        if min >= max {
            return min;
        }
        self.inner.gen_range(min..=max)
    }

    /// Uniform `u64` in `[min, max]` (inclusive).
    pub fn next_u64_inclusive(&mut self, min: u64, max: u64) -> u64 {
        if min >= max {
            return min;
        }
        self.inner.gen_range(min..=max)
    }

    /// Uniform `i64` in `[min, max]` (inclusive).
    pub fn next_i64_inclusive(&mut self, min: i64, max: i64) -> i64 {
        if min >= max {
            return min;
        }
        self.inner.gen_range(min..=max)
    }

    /// Uniform `u64` over the full integer range.
    pub fn next_u64(&mut self) -> u64 {
        self.inner.gen::<u64>()
    }

    /// Uniform `u32` over the full integer range.
    pub fn next_u32(&mut self) -> u32 {
        self.inner.gen::<u32>()
    }

    /// Uniform `usize` in `[min, max]` (inclusive).
    pub fn next_usize_inclusive(&mut self, min: usize, max: usize) -> usize {
        if min >= max {
            return min;
        }
        self.inner.gen_range(min..=max)
    }

    /// Uniform `f64` in `[min, max)`.
    pub fn next_f64_range(&mut self, min: f64, max: f64) -> f64 {
        debug_assert!(min <= max);
        if min >= max {
            return min;
        }
        min + self.next_f64() * (max - min)
    }

    /// Uniform `f32` in `[min, max)`.
    pub fn next_f32_range(&mut self, min: f32, max: f32) -> f32 {
        debug_assert!(min <= max);
        if min >= max {
            return min;
        }
        min + self.next_f32() * (max - min)
    }

    /// Random boolean with equal probability.
    pub fn next_bool(&mut self) -> bool {
        self.next_u64() & 1 == 0
    }

    /// Random boolean with probability `p` of returning `true` (`p` in `[0, 1]`).
    pub fn next_bool_probability(&mut self, p: f64) -> bool {
        self.next_f64() < p.clamp(0.0, 1.0)
    }

    /// Selects a uniformly random element from `items`.
    ///
    /// Returns `None` if `items` is empty.
    pub fn choose<'a, T>(&mut self, items: &'a [T]) -> Option<&'a T> {
        if items.is_empty() {
            return None;
        }
        let idx = self.inner.gen_range(0..items.len());
        Some(&items[idx])
    }

    /// Selects an element using integer weights (must be non-empty, sum > 0).
    pub fn choose_weighted<'a, T>(&mut self, items: &'a [T], weights: &[u32]) -> Option<&'a T> {
        if items.is_empty() || items.len() != weights.len() {
            return None;
        }
        let total: u64 = weights.iter().map(|&w| w as u64).sum();
        if total == 0 {
            return None;
        }
        let mut pick = self.inner.gen_range(0..total);
        for (item, &weight) in items.iter().zip(weights.iter()) {
            if weight == 0 {
                continue;
            }
            if pick < weight as u64 {
                return Some(item);
            }
            pick -= weight as u64;
        }
        items.last()
    }

    /// Selects an element using non-negative floating point weights.
    pub fn choose_weighted_f64<'a, T>(&mut self, items: &'a [T], weights: &[f64]) -> Option<&'a T> {
        if items.is_empty() || items.len() != weights.len() {
            return None;
        }
        let total: f64 = weights
            .iter()
            .copied()
            .filter(|w| w.is_finite() && *w > 0.0)
            .sum();
        if total <= 0.0 {
            return None;
        }
        let mut pick = self.next_f64_range(0.0, total);
        for (item, &weight) in items.iter().zip(weights.iter()) {
            if !weight.is_finite() || weight <= 0.0 {
                continue;
            }
            if pick < weight {
                return Some(item);
            }
            pick -= weight;
        }
        items.last()
    }

    /// Fills `buf` with pseudo-random bytes.
    pub fn fill_bytes(&mut self, buf: &mut [u8]) {
        self.inner.fill(buf);
    }
}

/// SplitMix64 — fast seed mixer for sub-stream derivation.
fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn hash_label(label: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in label.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_sequence() {
        let mut a = RngStream::new(42);
        let mut b = RngStream::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_f64(), b.next_f64());
        }
    }

    #[test]
    fn different_seeds_differ() {
        let mut a = RngStream::new(1);
        let mut b = RngStream::new(2);
        assert_ne!(a.next_f64(), b.next_f64());
    }

    #[test]
    fn derive_is_stable() {
        let base = RngStream::new(99);
        let d1 = base.derive("geology");
        let d2 = base.derive("geology");
        assert_eq!(d1.seed(), d2.seed());
    }

    #[test]
    fn weighted_choice_respects_weights() {
        let mut rng = RngStream::new(7);
        let items = ['a', 'b'];
        let weights = [100, 0];
        for _ in 0..10 {
            assert_eq!(*rng.choose_weighted(&items, &weights).unwrap(), 'a');
        }
    }

    #[test]
    fn range_is_in_bounds() {
        let mut rng = RngStream::new(123);
        for _ in 0..200 {
            let v = rng.next_f64_range(2.0, 5.0);
            assert!(v >= 2.0 && v < 5.0);
        }
    }
}
