//! Planetary evolution framework for WorldSmith.
//!
//! This crate hosts the long-term scientific simulation modules that evolve
//! planets from their initial formation state into mature, dynamic worlds.
//! Each discipline is an independent [`SimulationModule`] that plugs into
//! the existing engine scheduler.
//!
//! **Current modules:**
//! - [`core::CoreEvolutionModule`] — internal heat, cooling, and core state.
//! - [`mantle::MantleEvolutionModule`] — mantle thermal coupling.
//! - [`volcanism::VolcanismModule`] — surface volcanic activity.
//! - [`plate_tectonics::PlateTectonicsModule`] — plate motion and crustal recycling.
//! - [`atmosphere::AtmosphereModule`] — atmospheric evolution.
//! - [`hydrology::HydrologyModule`] — hydrosphere evolution.
//! - [`climate::ClimateModule`] — global climate energy balance.
//! - [`carbon_cycle::CarbonCycleModule`] — bulk carbon reservoir cycling.
//! - [`biosphere::BiosphereModule`] — planetary biomass and productivity.
//! - [`cryosphere::CryosphereModule`] — bulk planetary ice reservoirs.
//! - [`surface_chemistry::SurfaceChemistryModule`] — bulk planetary weathering and chemistry.
//! - [`habitability::HabitabilityModule`] — deterministic planetary habitability assessment.
//! - [`planet_classification::PlanetClassificationModule`] — deterministic planetary classification.
//!
//! **Planned modules (not yet implemented):**
//! - MagneticField.

pub mod atmosphere;
pub mod biosphere;
pub mod carbon_cycle;
pub mod climate;
pub mod core;
pub mod cryosphere;
pub mod habitability;
pub mod hydrology;
pub mod mantle;
pub mod planet_classification;
pub mod plate_tectonics;
pub mod plugin;
pub mod surface_chemistry;
pub mod volcanism;

pub use atmosphere::AtmosphereModule;
pub use biosphere::BiosphereModule;
pub use carbon_cycle::CarbonCycleModule;
pub use climate::ClimateModule;
pub use core::CoreEvolutionModule;
pub use cryosphere::CryosphereModule;
pub use habitability::HabitabilityModule;
pub use hydrology::HydrologyModule;
pub use mantle::MantleEvolutionModule;
pub use planet_classification::PlanetClassificationModule;
pub use plate_tectonics::PlateTectonicsModule;
pub use plugin::EvolutionPlugin;
pub use surface_chemistry::SurfaceChemistryModule;
pub use volcanism::VolcanismModule;
