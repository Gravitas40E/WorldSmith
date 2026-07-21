//! Engine-level foundations for WorldSmith.

pub mod builder;
pub mod config;
pub mod diagnostics;
pub mod engine;
pub mod error;
pub mod logging;
pub mod pipeline;
pub mod registry;
pub mod scheduler;
pub mod time;

pub use builder::EngineBuilder;
pub use config::{
    DebugSettings, EngineConfig, EngineSettings, RenderingSettings, SimulationSettings,
};
pub use diagnostics::EngineDiagnostics;
pub use engine::Engine;
pub use error::{EngineError, EngineResult};
pub use logging::{LogLevel, LogRecord, Logger};
pub use pipeline::{Pipeline, PipelineStageDescriptor};
pub use registry::{ModuleRegistry, RegisteredModule};
pub use scheduler::{Scheduler, SchedulerStats};
pub use time::SimulationClock;
