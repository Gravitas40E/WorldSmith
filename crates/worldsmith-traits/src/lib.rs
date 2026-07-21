//! Engine contracts used by WorldSmith simulation modules.
//!
//! This crate defines interfaces only. Implementations live in simulation,
//! engine, serialization, rendering, or tooling crates.

use serde::{de::DeserializeOwned, Serialize};
use worldsmith_state::{
    EventQueue, FieldKey, FieldRegistry, SimulationEvent, SimulationSnapshot, WorldState,
};

/// Error type returned by simulation contracts.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ContractError {
    /// Contract was given invalid configuration or state.
    #[error("invalid contract input: {0}")]
    InvalidInput(String),
    /// Module update failed.
    #[error("module error: {0}")]
    ModuleError(String),
}

/// Result type used by simulation contracts.
pub type ContractResult<T> = Result<T, ContractError>;

/// Immutable state access contract for simulation modules.
///
/// Readers may inspect the authoritative state but must not mutate it. Future
/// schedulers can use this trait to expose restricted state views.
pub trait StateReader {
    /// Returns the current immutable world state.
    fn world(&self) -> &WorldState;

    /// Returns the registered field vocabulary.
    fn field_registry(&self) -> &FieldRegistry {
        &self.world().field_registry
    }
}

/// Mutable state access contract for simulation modules.
///
/// Writers mutate only the authoritative `WorldState`; no hidden global state
/// or side-channel mutation is part of the contract.
pub trait StateWriter: StateReader {
    /// Returns mutable access to the authoritative world state.
    fn world_mut(&mut self) -> &mut WorldState;

    /// Returns mutable access to the deterministic event queue.
    fn event_queue_mut(&mut self) -> &mut EventQueue {
        &mut self.world_mut().event_queue
    }
}

/// Immutable context supplied to a module update.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModuleContext {
    /// Simulation timestamp at the start of the update in seconds.
    pub timestamp_s: f64,
    /// Delta time for this update in seconds.
    pub delta_seconds: f64,
    /// Current deterministic seed.
    pub seed: u64,
}

/// Simulation module lifecycle contract.
///
/// Modules declare field reads/writes and communicate changes through immutable
/// events. They should not call rendering, UI, networking, or uncontrolled RNG.
pub trait SimulationModule {
    /// Stable module identifier used by pipeline registration and save metadata.
    fn id(&self) -> &'static str;

    /// Human-readable module name.
    fn name(&self) -> &'static str {
        self.id()
    }

    /// Initializes module-local resources from the current state.
    fn initialize(&mut self, state: &mut dyn StateWriter) -> ContractResult<()>;

    /// Advances module behavior for one scheduled update.
    fn update(&mut self, context: ModuleContext, state: &mut dyn StateWriter)
        -> ContractResult<()>;

    /// Releases module-local resources.
    fn shutdown(&mut self, state: &mut dyn StateWriter) -> ContractResult<()>;

    /// Fields read by this module.
    fn reads(&self) -> Vec<FieldKey>;

    /// Fields written by this module.
    fn writes(&self) -> Vec<FieldKey>;

    /// Publishes queued module events after update.
    fn publish_events(&mut self) -> Vec<SimulationEvent>;

    /// Consumes immutable events emitted by earlier deterministic stages.
    fn consume_events(&mut self, events: &[SimulationEvent]) -> ContractResult<()>;
}

/// Pipeline stage contract for deterministic engine scheduling.
///
/// Stages declare an order key and dependencies so new modules can register
/// without hard-coded engine changes.
pub trait PipelineStage {
    /// Stable stage identifier.
    fn stage_id(&self) -> &str;

    /// Human-readable stage name.
    fn stage_name(&self) -> &str {
        self.stage_id()
    }

    /// Deterministic order key. Lower values execute earlier.
    fn order(&self) -> i32;

    /// Stage identifiers that must execute before this stage.
    fn dependencies(&self) -> Vec<String>;

    /// Whether this stage can execute for the provided state.
    fn is_enabled(&self, _state: &WorldState) -> bool {
        true
    }
}

/// Produces immutable snapshots for rendering, export, or inspection.
///
/// Snapshot consumers must not depend on mutable simulation internals.
pub trait SnapshotProducer {
    /// Snapshot type produced by this object.
    type Snapshot: Clone + Serialize + DeserializeOwned;

    /// Produces a snapshot of the current state.
    fn snapshot(&self) -> Self::Snapshot;
}

/// Validation contract for models, state, and configuration.
pub trait Validatable {
    /// Validates structural correctness.
    fn validate(&self) -> ContractResult<()>;
}

/// Scientific consistency contract for data that can be checked after creation.
///
/// This trait should report impossible or contradictory state without performing
/// simulation updates or deriving new values.
pub trait ScientificConsistency {
    /// Checks scientific consistency of existing values.
    fn check_consistency(&self) -> ContractResult<()>;
}

/// Configuration validation contract.
pub trait ConfigurationValidation {
    /// Validates configuration values before execution.
    fn validate_configuration(&self) -> ContractResult<()>;
}

/// Serialization-ready model marker and version hook.
///
/// Models implementing this trait can participate in save files, replay logs,
/// and future network serialization.
pub trait SerializableModel: Serialize + DeserializeOwned {
    /// Schema version for this model.
    fn schema_version(&self) -> u32 {
        1
    }
}

/// Human and tool inspection contract.
pub trait Inspectable {
    /// Stable display label.
    fn inspect_label(&self) -> String;

    /// Key-value details suitable for UI, CLI, or debug export.
    fn inspect_fields(&self) -> Vec<(String, String)>;
}

impl SnapshotProducer for WorldState {
    type Snapshot = SimulationSnapshot;

    fn snapshot(&self) -> Self::Snapshot {
        WorldState::snapshot(self)
    }
}

impl StateReader for WorldState {
    fn world(&self) -> &WorldState {
        self
    }
}

impl StateWriter for WorldState {
    fn world_mut(&mut self) -> &mut WorldState {
        self
    }
}
