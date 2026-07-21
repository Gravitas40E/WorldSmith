//! Engine diagnostics for developer tooling.

use serde::{Deserialize, Serialize};

use crate::scheduler::SchedulerStats;

/// Snapshot of engine runtime diagnostics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EngineDiagnostics {
    /// Registered module identifiers.
    pub registered_modules: Vec<String>,
    /// Active pipeline order.
    pub active_pipeline: Vec<String>,
    /// Number of outer ticks requested.
    pub tick_count: u64,
    /// Current simulation time in seconds.
    pub simulation_time_s: f64,
    /// Number of events waiting in the queue.
    pub queued_event_count: usize,
    /// Number of snapshots retained by the engine.
    pub snapshot_count: usize,
    /// Current master seed.
    pub current_seed: u64,
    /// Scheduler execution statistics.
    pub scheduler_stats: SchedulerStats,
    /// Whether the engine has been initialized.
    pub initialized: bool,
    /// Whether the engine is running.
    pub running: bool,
}
