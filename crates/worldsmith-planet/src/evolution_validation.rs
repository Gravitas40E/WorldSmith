//! Validation rules specific to the planetary evolution pipeline.
//!
//! This module validates evolution inputs and outputs for physical
//! plausibility, including age, stellar luminosity, and orbital
//! parameter ranges. Every constant has a physical justification.

use crate::errors::{PlanetFormationError, PlanetFormationResult};

/// Validates evolution input parameters.
///
/// # Physics Basis
///
/// - Age must be non-negative and finite (time is a forward arrow).
/// - Stellar luminosity must be positive (all real stars emit energy).
/// - Luminosity is clamped to physically modeled range (0.001–10,000 L☉).
pub fn validate_evolution_inputs(
    stellar_luminosity_solar: f64,
    age_gyr: f64,
) -> PlanetFormationResult<()> {
    if !stellar_luminosity_solar.is_finite() || stellar_luminosity_solar <= 0.0 {
        return Err(PlanetFormationError::InvalidEvolution(format!(
            "stellar luminosity must be positive and finite: got {}",
            stellar_luminosity_solar
        )));
    }
    if !(0.001..=10_000.0).contains(&stellar_luminosity_solar) {
        return Err(PlanetFormationError::InvalidEvolution(format!(
            "stellar luminosity {} L☉ outside modeled range [0.001, 10000]",
            stellar_luminosity_solar
        )));
    }
    if !age_gyr.is_finite() || age_gyr < 0.0 {
        return Err(PlanetFormationError::InvalidEvolution(format!(
            "age must be non-negative and finite: got {} Gyr",
            age_gyr
        )));
    }
    if age_gyr > 13.8e3 {
        return Err(PlanetFormationError::InvalidEvolution(format!(
            "age {} Gyr exceeds the age of the universe",
            age_gyr
        )));
    }
    Ok(())
}

/// Validates that a planet's mass and age are compatible for the evolution
/// pipeline to produce physically meaningful results.
pub fn validate_planet_for_evolution(
    mass_kg: f64,
    radius_m: f64,
    orbital_au: f64,
) -> PlanetFormationResult<()> {
    if !mass_kg.is_finite() || mass_kg <= 0.0 {
        return Err(PlanetFormationError::InvalidEvolution(
            "planet mass must be positive and finite for evolution".to_string(),
        ));
    }
    if !radius_m.is_finite() || radius_m <= 0.0 {
        return Err(PlanetFormationError::InvalidEvolution(
            "planet radius must be positive and finite for evolution".to_string(),
        ));
    }
    if !orbital_au.is_finite() || orbital_au <= 0.0 {
        return Err(PlanetFormationError::InvalidEvolution(
            "orbital distance must be positive and finite for evolution".to_string(),
        ));
    }
    // Mass cannot exceed ~13 Jupiter masses without deuterium fusion (brown dwarf boundary)
    let max_kg = 13.0 * 1.898e27; // 13 Jupiter masses
    if mass_kg > max_kg {
        return Err(PlanetFormationError::InvalidEvolution(format!(
            "planet mass {:.3e} kg exceeds ~13 M_Jup deuterium fusion boundary",
            mass_kg
        )));
    }
    // Orbital distance must be within reasonable range (0.01 AU to 1000 AU)
    if !(0.01..=1000.0).contains(&orbital_au) {
        return Err(PlanetFormationError::InvalidEvolution(format!(
            "orbital distance {:.2} AU outside modeled range [0.01, 1000]",
            orbital_au
        )));
    }
    Ok(())
}

/// Validates that a rotation period is within physically modeled bounds.
pub fn validate_rotation(rotation_period_s: Option<f64>) -> PlanetFormationResult<()> {
    if let Some(period) = rotation_period_s {
        if !period.is_finite() || period <= 0.0 {
            return Err(PlanetFormationError::InvalidEvolution(format!(
                "rotation period must be positive and finite: got {} s",
                period
            )));
        }
        // Rotation period cannot be faster than structural limits (~1 hour for rocky bodies)
        if period < 3_600.0 {
            return Err(PlanetFormationError::InvalidEvolution(format!(
                "rotation period {:.0} s is below ~1 hour structural limit",
                period
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use worldsmith_math::constants;

    #[test]
    fn valid_earth_like_passes() {
        assert!(validate_evolution_inputs(1.0, 4.5).is_ok());
        assert!(
            validate_planet_for_evolution(constants::EARTH_MASS, constants::EARTH_RADIUS, 1.0)
                .is_ok()
        );
        assert!(validate_rotation(Some(86_400.0)).is_ok());
    }

    #[test]
    fn zero_luminosity_fails() {
        assert!(validate_evolution_inputs(0.0, 4.5).is_err());
    }

    #[test]
    fn nan_luminosity_fails() {
        assert!(validate_evolution_inputs(f64::NAN, 4.5).is_err());
    }

    #[test]
    fn negative_age_fails() {
        assert!(validate_evolution_inputs(1.0, -1.0).is_err());
    }

    #[test]
    fn too_old_age_fails() {
        assert!(validate_evolution_inputs(1.0, 14_000.0).is_err());
    }

    #[test]
    fn luminosity_out_of_range_fails() {
        assert!(validate_evolution_inputs(0.0001, 4.5).is_err());
        assert!(validate_evolution_inputs(50_000.0, 4.5).is_err());
    }

    #[test]
    fn zero_mass_fails() {
        assert!(validate_planet_for_evolution(0.0, constants::EARTH_RADIUS, 1.0).is_err());
    }

    #[test]
    fn zero_radius_fails() {
        assert!(validate_planet_for_evolution(constants::EARTH_MASS, 0.0, 1.0).is_err());
    }

    #[test]
    fn brown_dwarf_boundary_fails() {
        let jupiter_mass = 1.898e27;
        assert!(validate_planet_for_evolution(14.0 * jupiter_mass, 7.0e7, 5.0).is_err());
    }

    #[test]
    fn extreme_orbital_distance_fails() {
        assert!(validate_planet_for_evolution(
            constants::EARTH_MASS,
            constants::EARTH_RADIUS,
            0.001
        )
        .is_err());
        assert!(validate_planet_for_evolution(
            constants::EARTH_MASS,
            constants::EARTH_RADIUS,
            2_000.0
        )
        .is_err());
    }

    #[test]
    fn too_fast_rotation_fails() {
        assert!(validate_rotation(Some(1_000.0)).is_err());
    }

    #[test]
    fn none_rotation_passes() {
        assert!(validate_rotation(None).is_ok());
    }

    #[test]
    fn negative_rotation_fails() {
        assert!(validate_rotation(Some(-100.0)).is_err());
    }

    #[test]
    fn nan_rotation_fails() {
        assert!(validate_rotation(Some(f64::NAN)).is_err());
    }
}
