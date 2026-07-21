//! Deterministic seed-based random number generation for WorldSmith simulations.
//!
//! All randomness flows through [`RngStream`]. Simulations must never use
//! thread-local or global RNG sources to preserve reproducibility.

mod stream;

pub use stream::RngStream;
