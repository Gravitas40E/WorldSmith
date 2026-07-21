//! Deterministic protoplanetary disk and planet formation models.
//!
//! This crate models how planets emerge from disk structure, condensation,
//! planetesimal growth, accretion, and simplified migration. It does not model
//! atmospheres, climate, geology, terrain, rendering, or arbitrary templates.
//!
//! ## Phase 7 — Planet Evolution
//!
//! The evolution pipeline (see [`evolution::evolve_planet`]) transforms formed
//! planets into complete geophysical worlds with:
//!
//! - Interior differentiation (core, mantle, crust, magnetic field)
//! - Atmosphere generation (outgassing, retention, composition)
//! - Climate feedback (equilibrium temperature, greenhouse, ice, humidity)
//! - Weather system (winds, precipitation, storms)
//! - Hydrology (ocean condensation)
//! - Geological evolution (tectonics, volcanism, erosion)
//! - Habitability assessment
//! - Scientific timeline

pub mod accretion;
pub mod builder;
pub mod classification;
pub mod climate_feedback;
pub mod condensation;
pub mod density;
pub mod disk;
pub mod embryo;
pub mod errors;
pub mod evolution;
pub mod evolution_validation;
pub mod geological_evolution;
pub mod habitability;
pub mod hydrology;
pub mod interior;
pub mod migration;
pub mod module;
pub mod planetesimal;
pub mod report;
pub mod scientific;
pub mod temperature;
pub mod validation;
pub mod weather;

pub use accretion::{accrete_planetesimals, AccretionEvent, AccretionSummary};
pub use builder::{FormationConfig, PlanetFormationBuilder, PlanetFormationOutput};
pub use classification::classify_embryo;
pub use climate_feedback::{compute_climate_feedback, ClimateFeedback};
pub use condensation::{available_materials, CondensedMaterial, DiskRegion};
pub use density::{midplane_pressure_pa, surface_density_kg_m2};
pub use disk::{disk_regions, DiskRegionBounds, ProtoplanetaryDisk};
pub use embryo::{PlanetaryEmbryo, PlanetaryEmbryoComposition};
pub use errors::{PlanetFormationError, PlanetFormationResult as Result};
pub use evolution::{evolve_planet, EvolutionTimelineEntry, PlanetEvolutionOutput};
pub use evolution_validation::{
    validate_evolution_inputs, validate_planet_for_evolution, validate_rotation,
};
pub use geological_evolution::{compute_geological_evolution, GeologicalEvolution};
pub use habitability::assess_habitability;
pub use hydrology::derive_ocean_properties;
pub use interior::{differentiate_interior, HeatBudget, InteriorModel};
pub use migration::{migrate_embryo, MigrationModel, MigrationRecord};
pub use module::{
    PlanetEvolutionModule, PlanetEvolutionModuleConfig, PlanetFormationModule,
    PlanetFormationModuleConfig,
};
pub use planetesimal::{generate_planetesimals, Planetesimal};
pub use report::{PlanetEvolutionReport, PlanetFormationReport};
pub use scientific::check_planet_consistency;
pub use temperature::disk_temperature_k;
pub use weather::{derive_weather_system, PrecipitationRegime, StormIntensity, WeatherSystem};
