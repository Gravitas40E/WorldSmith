//! Reusable engine error types.

use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum EngineError {
    #[error("invalid configuration: {0}")]
    InvalidConfiguration(String),
    #[error("invalid orbit: {0}")]
    InvalidOrbit(String),
    #[error("simulation error: {0}")]
    SimulationError(String),
    #[error("module error: {0}")]
    ModuleError(String),
    #[error("duplicate module registered: {0}")]
    DuplicateModule(String),
    #[error("missing dependency `{dependency}` required by `{module}`")]
    MissingDependency { module: String, dependency: String },
    #[error("circular pipeline dependency involving: {0}")]
    CircularDependency(String),
    #[error("engine lifecycle error: {0}")]
    Lifecycle(String),
}

pub type EngineResult<T> = Result<T, EngineError>;

impl From<worldsmith_traits::ContractError> for EngineError {
    fn from(value: worldsmith_traits::ContractError) -> Self {
        Self::ModuleError(value.to_string())
    }
}
