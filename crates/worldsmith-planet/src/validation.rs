//! Validation helpers for disk and formation inputs.

use crate::errors::{PlanetFormationError, PlanetFormationResult};

/// Validates a positive finite disk mass.
pub fn validate_disk_mass_kg(mass_kg: f64) -> PlanetFormationResult<()> {
    if !mass_kg.is_finite() || mass_kg <= 0.0 {
        return Err(PlanetFormationError::InvalidDiskMass(
            "disk mass must be positive and finite".to_string(),
        ));
    }
    Ok(())
}

/// Validates a positive finite radius or orbital distance.
pub fn validate_positive_radius_m(radius_m: f64) -> PlanetFormationResult<()> {
    if !radius_m.is_finite() || radius_m <= 0.0 {
        return Err(PlanetFormationError::InvalidRadius(
            "radius must be positive and finite".to_string(),
        ));
    }
    Ok(())
}

/// Validates that composition fractions are finite, non-negative, and not above one.
pub fn validate_fraction(value: f64, name: &str) -> PlanetFormationResult<()> {
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        return Err(PlanetFormationError::InvalidComposition(format!(
            "{name} must be a finite fraction in [0, 1]"
        )));
    }
    Ok(())
}
