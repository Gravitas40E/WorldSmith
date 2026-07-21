//! Validation helpers for stellar inputs.

use crate::errors::{StellarError, StellarResult};

/// Minimum supported stellar mass in solar masses for these approximations.
pub const MIN_SUPPORTED_MASS_SOLAR: f64 = 0.08;
/// Maximum supported stellar mass in solar masses before massive-star models are needed.
pub const MAX_SUPPORTED_MASS_SOLAR: f64 = 50.0;

/// Validates stellar mass in solar masses.
pub fn validate_mass_solar(mass_solar: f64) -> StellarResult<()> {
    if !mass_solar.is_finite() || mass_solar <= 0.0 {
        return Err(StellarError::InvalidMass(
            "mass must be positive and finite".to_string(),
        ));
    }
    if !(MIN_SUPPORTED_MASS_SOLAR..=MAX_SUPPORTED_MASS_SOLAR).contains(&mass_solar) {
        return Err(StellarError::InvalidMass(format!(
            "mass {mass_solar} M_sun is outside the supported range {MIN_SUPPORTED_MASS_SOLAR}..={MAX_SUPPORTED_MASS_SOLAR}"
        )));
    }
    Ok(())
}

/// Validates stellar age in gigayears.
pub fn validate_age_gyr(age_gyr: f64) -> StellarResult<()> {
    if !age_gyr.is_finite() || age_gyr < 0.0 {
        return Err(StellarError::InvalidAge(
            "age must be finite and non-negative".to_string(),
        ));
    }
    Ok(())
}

/// Validates mass fraction metallicity.
pub fn validate_metallicity(metallicity: f64) -> StellarResult<()> {
    if !metallicity.is_finite() || metallicity < 0.0 {
        return Err(StellarError::InvalidMetallicity(
            "metallicity must be finite and non-negative".to_string(),
        ));
    }
    Ok(())
}

/// Validates rotation period in days.
pub fn validate_rotation_days(rotation_days: Option<f64>) -> StellarResult<()> {
    if let Some(days) = rotation_days {
        if !days.is_finite() || days <= 0.0 {
            return Err(StellarError::InvalidRotation(
                "rotation period must be positive and finite".to_string(),
            ));
        }
    }
    Ok(())
}
