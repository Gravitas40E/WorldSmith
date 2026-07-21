//! Error types returned by stellar calculations and builders.

use thiserror::Error;

/// Result type for deterministic stellar calculations.
pub type StellarResult<T> = Result<T, StellarError>;

/// Descriptive stellar validation and calculation errors.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum StellarError {
    /// Stellar mass must be positive and within the supported approximation range.
    #[error("invalid stellar mass: {0}")]
    InvalidMass(String),
    /// Stellar age must be finite and non-negative.
    #[error("invalid stellar age: {0}")]
    InvalidAge(String),
    /// Metallicity must be finite and non-negative.
    #[error("invalid metallicity: {0}")]
    InvalidMetallicity(String),
    /// Rotation period must be finite and positive when provided.
    #[error("invalid rotation period: {0}")]
    InvalidRotation(String),
    /// Effective temperature is outside supported stellar classification ranges.
    #[error("invalid stellar temperature: {0}")]
    InvalidTemperature(String),
}
