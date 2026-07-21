//! Simplified stellar evolution stages.

use serde::{Deserialize, Serialize};

use crate::equations::mass_luminosity_solar;

/// Simplified evolutionary stage used by early stellar simulations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StellarEvolutionStage {
    /// Collapsing pre-main-sequence object.
    Protostar,
    /// Hydrogen-burning main-sequence star.
    MainSequence,
    /// Post-main-sequence envelope expansion has begun.
    Subgiant,
    /// Red giant branch approximation.
    RedGiant,
    /// Compact white dwarf remnant.
    WhiteDwarf,
}

/// Estimates main-sequence lifetime in gigayears.
///
/// Approximation: available fuel scales with mass while luminosity controls
/// consumption, `t_ms ~= 10 Gyr * M/L`.
pub fn main_sequence_lifetime_gyr(mass_solar: f64) -> f64 {
    10.0 * mass_solar / mass_luminosity_solar(mass_solar)
}

/// Determines simplified evolutionary stage from mass and age.
pub fn evolution_stage(mass_solar: f64, age_gyr: f64) -> StellarEvolutionStage {
    if age_gyr < 0.01 {
        return StellarEvolutionStage::Protostar;
    }
    let lifetime = main_sequence_lifetime_gyr(mass_solar);
    if age_gyr <= lifetime {
        StellarEvolutionStage::MainSequence
    } else if age_gyr <= lifetime * 1.10 {
        StellarEvolutionStage::Subgiant
    } else if age_gyr <= lifetime * 1.25 {
        StellarEvolutionStage::RedGiant
    } else {
        StellarEvolutionStage::WhiteDwarf
    }
}
