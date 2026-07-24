//! Deterministic scientific validation framework for the WorldSmith Planet
//! Evolution subsystem.
//!
//! This crate provides reusable validation helpers and regression tests for
//! the Phase 10 planetary evolution modules (`CoreEvolutionModule`,
//! `MantleEvolutionModule`, `VolcanismModule`, and `PlateTectonicsModule`).
//! No new planetary physics is introduced here; this crate only observes,
//! measures, and asserts.
//!
//! ## Validation categories
//!
//! - **State validation** — temperatures remain finite, no NaN/Inf, optional
//!   state consistency, enum validity.
//! - **Ownership validation** — each field has exactly one declared runtime
//!   writer; duplicate ownership is detected.
//! - **Scientific invariants** — `core_temperature >= mantle_temperature`,
//!   non-negative fluxes/rates, etc.
//! - **Long-term stability** — run 100 / 1,000 / 10,000 / 100,000 ticks and
//!   verify no divergence, no exploding values, no oscillation, no NaN.
//! - **Deterministic replay / golden simulation** — identical seed, state,
//!   and timestep sequence must produce bit-for-bit identical outputs.
//! - **Cross-module validation** — no module writes another module's fields;
//!   no cyclic dependencies.
//! - **Performance validation** — measure tick time, memory, and allocation
//!   counts at 100 / 1,000 / 10,000 / 100,000 planets.
//!
//! ## Philosophy
//!
//! Validation ≠ scientific realism.  We validate structural correctness,
//! determinism, and consistency.  Whether the equations match reality is a
//! separate scientific review task.
//!
//! ## Crate layout
//!
//! - [`state`] — finite-value and structural state checks.
//! - [`ownership`] — declared `reads()` / `writes()` analysis.
//! - [`invariants`] — scientific inequality checks.
//! - [`stability`] — long-run and stress-test harnesses.
//! - [`replay`] — deterministic replay + golden-world helpers.
//! - [`performance`] — tick-time, memory, and allocation counters.
//! - [`cross_module`] — dependency graph + field ownership checks.

pub mod cross_module;
pub mod invariants;
pub mod ownership;
pub mod performance;
pub mod replay;
pub mod stability;
pub mod state;

pub use cross_module::{validate_dependency_graph, validate_no_cross_module_writes};
pub use invariants::{validate_scientific_invariants, ScientificInvariantError};
pub use ownership::{validate_field_ownership, OwnershipError};
pub use performance::{measure_performance, PerformanceReport};
pub use replay::{deterministic_replay, GoldenWorld, ReplayOutcome};
pub use stability::{run_long_run_stability, StabilityReport};
pub use state::{validate_state, StateValidationError};
