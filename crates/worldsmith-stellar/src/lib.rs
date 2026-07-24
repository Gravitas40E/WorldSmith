//! Stellar physics and simplified evolution for WorldSmith.
//!
//! This crate provides deterministic, explainable stellar calculations. It does
//! not generate planets, render stars, or perform hidden random sampling.

pub mod blackbody;
pub mod builder;
pub mod classification;
pub mod equations;
pub mod errors;
pub mod evolution;
pub mod habitable_zone;
pub mod module;
pub mod orbital_module;
pub mod radiation;
pub mod report;
pub mod validation;

pub use blackbody::{blackbody_profile, ApproximateColor, BlackbodyProfile, RgbColor};
pub use builder::{StarBuilder, StellarProfile};
pub use classification::{classify_star, LuminosityClass, SpectralClassification};
pub use equations::{
    escape_velocity_m_s, mass_luminosity_solar, mass_radius_solar, stellar_flux_w_m2,
    surface_gravity_m_s2,
};
pub use errors::{StellarError, StellarResult};
pub use evolution::{evolution_stage, main_sequence_lifetime_gyr, StellarEvolutionStage};
pub use habitable_zone::{frost_lines, habitable_zone, FrostLines, HabitableZone};
pub use module::{StellarModule, StellarModuleConfig};
pub use radiation::{radiation_profile, StellarActivity, StellarRadiation};
pub use report::StellarReport;
