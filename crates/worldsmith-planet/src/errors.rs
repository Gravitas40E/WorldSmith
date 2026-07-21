//! Errors returned by planet formation calculations.

use thiserror::Error;

/// Result type for planet formation operations.
pub type PlanetFormationResult<T> = Result<T, PlanetFormationError>;

/// Descriptive validation and simulation errors.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum PlanetFormationError {
    /// Disk mass was impossible or unsupported.
    #[error("invalid disk mass: {0}")]
    InvalidDiskMass(String),
    /// Radius or orbital distance was invalid.
    #[error("invalid radius or orbital distance: {0}")]
    InvalidRadius(String),
    /// Composition fractions were impossible.
    #[error("invalid composition: {0}")]
    InvalidComposition(String),
    /// Duplicate identifier was encountered.
    #[error("duplicate identifier: {0}")]
    DuplicateIdentifier(String),
    /// Required stellar data was missing.
    #[error("missing stellar data: {0}")]
    MissingStellarData(String),
    /// Planet evolution could not be evaluated.
    #[error("invalid planet evolution input: {0}")]
    InvalidEvolution(String),
}
