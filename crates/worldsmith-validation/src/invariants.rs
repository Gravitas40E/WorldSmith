//! Scientific invariant validators.
//!
//! Check physical inequalities that must hold for a structurally sound
//! planetary simulation independent of whether the values are "realistic".

use std::collections::BTreeMap;

use worldsmith_models::{Planet, PlanetId};

/// Errors detected during invariant validation.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ScientificInvariantError {
    /// Core temperature is lower than mantle temperature.
    #[error("planet {planet_id}: core_temperature ({core_temperature}) must be >= mantle_temperature ({mantle_temperature})")]
    CoreBelowMantle {
        /// Planet identifier.
        planet_id: String,
        /// Core temperature.
        core_temperature: f64,
        /// Mantle temperature.
        mantle_temperature: f64,
    },
    /// Volcanic flux is negative.
    #[error("planet {planet_id}: volcanic_flux must be non-negative, got {volcanic_flux}")]
    NegativeVolcanicFlux {
        /// Planet identifier.
        planet_id: String,
        /// Volcanic flux.
        volcanic_flux: f64,
    },
    /// Plate velocity is negative.
    #[error("planet {planet_id}: plate_velocity must be non-negative, got {plate_velocity}")]
    NegativePlateVelocity {
        /// Planet identifier.
        planet_id: String,
        /// Plate velocity.
        plate_velocity: f64,
    },
    /// Crustal recycling rate is negative.
    #[error("planet {planet_id}: crustal_recycling_rate must be non-negative, got {crustal_recycling_rate}")]
    NegativeCrustalRecyclingRate {
        /// Planet identifier.
        planet_id: String,
        /// Crustal recycling rate.
        crustal_recycling_rate: f64,
    },
    /// Radiogenic heat is negative.
    #[error("planet {planet_id}: radiogenic_heat must be non-negative, got {radiogenic_heat}")]
    NegativeRadiogenicHeat {
        /// Planet identifier.
        planet_id: String,
        /// Radiogenic heat.
        radiogenic_heat: f64,
    },
    /// Internal heat is negative.
    #[error("planet {planet_id}: internal_heat must be non-negative, got {internal_heat}")]
    NegativeInternalHeat {
        /// Planet identifier.
        planet_id: String,
        /// Internal heat.
        internal_heat: f64,
    },
}

/// Validates scientific invariants across all planets in `WorldState`.
pub fn validate_scientific_invariants(
    planets: &BTreeMap<PlanetId, Planet>,
) -> Result<(), ScientificInvariantError> {
    for (id, planet) in planets.iter() {
        let planet_id = format!("{id:?}");
        if let Some(interior) = &planet.interior {
            if interior.core_temperature < interior.mantle_temperature {
                return Err(ScientificInvariantError::CoreBelowMantle {
                    planet_id,
                    core_temperature: interior.core_temperature,
                    mantle_temperature: interior.mantle_temperature,
                });
            }
            if interior.radiogenic_heat < 0.0 {
                return Err(ScientificInvariantError::NegativeRadiogenicHeat {
                    planet_id,
                    radiogenic_heat: interior.radiogenic_heat,
                });
            }
            if interior.internal_heat < 0.0 {
                return Err(ScientificInvariantError::NegativeInternalHeat {
                    planet_id,
                    internal_heat: interior.internal_heat,
                });
            }
        }
        if let Some(volcanism) = &planet.volcanism {
            if volcanism.volcanic_flux < 0.0 {
                return Err(ScientificInvariantError::NegativeVolcanicFlux {
                    planet_id,
                    volcanic_flux: volcanism.volcanic_flux,
                });
            }
        }
        if let Some(plate_tectonics) = &planet.plate_tectonics {
            if plate_tectonics.plate_velocity < 0.0 {
                return Err(ScientificInvariantError::NegativePlateVelocity {
                    planet_id,
                    plate_velocity: plate_tectonics.plate_velocity,
                });
            }
            if plate_tectonics.crustal_recycling_rate < 0.0 {
                return Err(ScientificInvariantError::NegativeCrustalRecyclingRate {
                    planet_id,
                    crustal_recycling_rate: plate_tectonics.crustal_recycling_rate,
                });
            }
        }
    }
    Ok(())
}
