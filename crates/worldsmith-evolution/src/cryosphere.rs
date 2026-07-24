//! Planetary cryosphere: deterministic bulk ice reservoirs and fluxes.
//!
//! This module implements a V1 deterministic planetary cryosphere model.
//! It models continental ice, sea ice, and snow at a planetary scale.
//! No glacier geometry, ice dynamics, or seasonal snow cover models are
//! implemented in V1.
//!
//! ## Responsibilities
//! - Owns `continental_ice_mass_kg`, `sea_ice_mass_kg`, `snow_mass_kg`,
//!   `permanent_ice_fraction`, `seasonal_snow_fraction`, `melt_rate_kg_per_s`,
//!   `freeze_rate_kg_per_s`, `planetary_ice_fraction`,
//!   `cryosphere_albedo_modifier`, and `sea_level_offset_m` per ADR-011.
//! - Reads `ClimateState`, `HydrologyState`, and `Planet` properties after
//!   `worldsmith.evolution.biosphere`.
//!
//! ## Simplifying assumptions
//! 1. **Bulk ice reservoirs**: a single continental ice mass and sea ice mass.
//!    No ice sheets, glaciers, or spatial distributions.
//! 2. **Temperature-driven phase change**: freezing and melting are triggered
//!    by a configurable surface temperature threshold relative to the climate
//!    equilibrium temperature.
//! 3. **Water-limited ice growth**: freeze rate is clamped by available water
//!    from `HydrologyState`.
//! 4. **Static albedo modifier**: ice albedo is a constant; no surface aging
//!    or impurity evolution.
//! 5. **No ice dynamics**: no ice flow, calving, or glacial isostatic
//!    adjustment.
//! 6. **No seasonal cycles**: seasonal snow fraction is a deterministic
//!    function of surface temperature, not a time-dependent cycle.
//!
//! ## Future extensions
//! - dynamic glacier flow
//! - seasonal snow cover cycles
//! - climate feedback via planetary albedo
//! - sea-level change coupling
//!
//! ## Ownership
//!
//! - **Reads**: `ClimateState`, `HydrologyState`, `Planet` properties
//! - **Writes**: `continental_ice_mass_kg`, `sea_ice_mass_kg`,
//!   `snow_mass_kg`, `permanent_ice_fraction`, `seasonal_snow_fraction`,
//!   `melt_rate_kg_per_s`, `freeze_rate_kg_per_s`, `planetary_ice_fraction`,
//!   `cryosphere_albedo_modifier`, `sea_level_offset_m`
//! - **Never modifies**: `ClimateState`, `HydrologyState`, `BiosphereState`,
//!   `CarbonCycleState`, `AtmosphereState`, `InteriorState`, `VolcanismState`,
//!   `PlateTectonicsState`, `climate`, `ocean`, `magnetic_field`,
//!   `habitability`

use serde::{Deserialize, Serialize};
use worldsmith_models::{
    AtmosphereState, ClimateState, CryosphereState, HabitabilityState, HydrologyState, Planet,
    PlanetClassificationState, PlanetId, SurfaceChemistryState,
};
use worldsmith_state::{FieldKey, SimulationEvent};
use worldsmith_traits::{ContractResult, ModuleContext, SimulationModule, StateWriter};

const DEFAULT_FREEZING_TEMPERATURE: f64 = 273.15;
const DEFAULT_MELTING_TEMPERATURE: f64 = 273.15;
const DEFAULT_FREEZE_RATE: f64 = 1.0e9;
const DEFAULT_MELT_RATE: f64 = 1.0e9;
const DEFAULT_ICE_ALBEDO: f64 = 0.6;
const DEFAULT_TEMPERATURE_WIDTH: f64 = 5.0;
const DEFAULT_CONTINENTAL_ICE_MASS_KG: f64 = 2.0e18;
const DEFAULT_SEA_ICE_MASS_KG: f64 = 1.0e16;
const DEFAULT_SNOW_MASS_KG: f64 = 1.0e15;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CryosphereConfig {
    pub freezing_temperature: f64,
    pub melting_temperature: f64,
    pub freeze_rate: f64,
    pub melt_rate: f64,
    pub ice_albedo: f64,
    pub temperature_width: f64,
}

impl Default for CryosphereConfig {
    fn default() -> Self {
        Self {
            freezing_temperature: DEFAULT_FREEZING_TEMPERATURE,
            melting_temperature: DEFAULT_MELTING_TEMPERATURE,
            freeze_rate: DEFAULT_FREEZE_RATE,
            melt_rate: DEFAULT_MELT_RATE,
            ice_albedo: DEFAULT_ICE_ALBEDO,
            temperature_width: DEFAULT_TEMPERATURE_WIDTH,
        }
    }
}

pub struct CryosphereModule {
    config: CryosphereConfig,
    initialized: bool,
}

impl CryosphereModule {
    pub fn new(config: CryosphereConfig) -> Self {
        Self {
            config,
            initialized: false,
        }
    }

    fn initialize_reservoirs(&self, planet: &Planet) -> CryosphereState {
        let mut state = CryosphereState::default();
        state.continental_ice_mass_kg = DEFAULT_CONTINENTAL_ICE_MASS_KG;
        state.sea_ice_mass_kg = DEFAULT_SEA_ICE_MASS_KG;
        state.snow_mass_kg = DEFAULT_SNOW_MASS_KG;
        if let Some(existing) = &planet.cryosphere_state {
            state.continental_ice_mass_kg = existing.continental_ice_mass_kg;
            state.sea_ice_mass_kg = existing.sea_ice_mass_kg;
            state.snow_mass_kg = existing.snow_mass_kg;
            state.permanent_ice_fraction = existing.permanent_ice_fraction;
            state.seasonal_snow_fraction = existing.seasonal_snow_fraction;
            state.melt_rate_kg_per_s = existing.melt_rate_kg_per_s;
            state.freeze_rate_kg_per_s = existing.freeze_rate_kg_per_s;
        }
        state
    }

    fn compute_cryosphere(
        &self,
        cryo: &mut CryosphereState,
        climate: &ClimateState,
        hydro: &HydrologyState,
        _planet: &Planet,
    ) {
        let temp = climate.equilibrium_temperature_k + climate.greenhouse_temperature_offset_k;

        let freeze_suitability = ((self.config.freezing_temperature - temp)
            / self.config.temperature_width)
            .clamp(0.0, 1.0);
        let melt_suitability = ((temp - self.config.melting_temperature)
            / self.config.temperature_width)
            .clamp(0.0, 1.0);

        let _available_water = hydro.liquid_water_fraction * hydro.total_water_mass_kg;
        let freeze_capacity = self.config.freeze_rate * freeze_suitability;
        let melt_amount = self.config.melt_rate * melt_suitability;

        let mut continental = cryo.continental_ice_mass_kg;
        let mut sea_ice = cryo.sea_ice_mass_kg;
        let mut snow = cryo.snow_mass_kg;

        if freeze_capacity > 0.0 {
            let added = (freeze_capacity * 1.0).min(_available_water * 0.01);
            continental += added;
            snow += added * 0.3;
        }

        let melt = melt_amount.min(continental + sea_ice + snow);
        continental = (continental - melt * 0.6).max(0.0);
        sea_ice = (sea_ice - melt * 0.2).max(0.0);
        snow = (snow - melt * 0.2).max(0.0);

        continental = continental.max(0.0);
        sea_ice = sea_ice.max(0.0);
        snow = snow.max(0.0);

        let total_water = hydro.total_water_mass_kg + 1e-6;
        let total_ice = continental + sea_ice + snow;
        let planetary_ice_fraction = (total_ice / total_water).clamp(0.0, 1.0);

        cryo.continental_ice_mass_kg = continental;
        cryo.sea_ice_mass_kg = sea_ice;
        cryo.snow_mass_kg = snow;
        cryo.permanent_ice_fraction = (planetary_ice_fraction * 0.8).clamp(0.0, 1.0);
        cryo.seasonal_snow_fraction = (freeze_suitability * 0.3).clamp(0.0, 1.0);
        cryo.melt_rate_kg_per_s = melt_amount * melt_suitability;
        cryo.freeze_rate_kg_per_s = freeze_capacity * freeze_suitability;
        cryo.planetary_ice_fraction = planetary_ice_fraction;
        cryo.cryosphere_albedo_modifier =
            (planetary_ice_fraction * self.config.ice_albedo).clamp(0.0, 1.0);
        cryo.sea_level_offset_m = (-(melt * 1.0 / 1.0e15)).clamp(-100.0, 100.0);
    }

    fn tick(
        &self,
        mut cryo: CryosphereState,
        climate: &ClimateState,
        hydro: &HydrologyState,
        planet: &Planet,
    ) -> CryosphereState {
        self.compute_cryosphere(&mut cryo, climate, hydro, planet);
        cryo
    }
}

impl Default for CryosphereModule {
    fn default() -> Self {
        Self::new(CryosphereConfig::default())
    }
}

impl SimulationModule for CryosphereModule {
    fn id(&self) -> &'static str {
        "worldsmith.evolution.cryosphere"
    }

    fn initialize(&mut self, state: &mut dyn StateWriter) -> ContractResult<()> {
        let planet_ids: Vec<_> = state.world().planets.keys().cloned().collect();
        for planet_id in planet_ids {
            if let Some(planet) = state.world().planets.get(&planet_id).cloned() {
                let cryo = self.initialize_reservoirs(&planet);
                let mut updated = planet.clone();
                updated.cryosphere_state = Some(cryo);
                state.world_mut().planets.insert(updated.id, updated);
            }
        }
        self.initialized = true;
        Ok(())
    }

    fn update(
        &mut self,
        _context: ModuleContext,
        state: &mut dyn StateWriter,
    ) -> ContractResult<()> {
        if !self.initialized {
            return Ok(());
        }

        let snapshot: Vec<(PlanetId, Planet, Option<CryosphereState>)> = state
            .world()
            .planets
            .values()
            .map(|planet| (planet.id, planet.clone(), planet.cryosphere_state.clone()))
            .collect();

        for (_planet_id, planet, cryo) in snapshot {
            let cryo = match cryo {
                Some(cryo) => cryo,
                None => continue,
            };

            let climate = match &planet.climate_state {
                Some(c) => c,
                None => continue,
            };
            let hydro = match &planet.hydrology_state {
                Some(h) => h,
                None => continue,
            };

            let updated = self.tick(cryo, climate, hydro, &planet);
            let mut updated_planet = planet;
            updated_planet.cryosphere_state = Some(updated);
            state
                .world_mut()
                .planets
                .insert(updated_planet.id, updated_planet);
        }

        Ok(())
    }

    fn reads(&self) -> Vec<FieldKey> {
        vec![
            FieldKey::ContinentalIceMass,
            FieldKey::SeaIceMass,
            FieldKey::SnowMass,
            FieldKey::PermanentIceFraction,
            FieldKey::SeasonalSnowFraction,
            FieldKey::MeltRate,
            FieldKey::FreezeRate,
            FieldKey::PlanetaryIceFraction,
            FieldKey::CryosphereAlbedoModifier,
            FieldKey::SeaLevelOffset,
        ]
    }

    fn writes(&self) -> Vec<FieldKey> {
        vec![
            FieldKey::ContinentalIceMass,
            FieldKey::SeaIceMass,
            FieldKey::SnowMass,
            FieldKey::PermanentIceFraction,
            FieldKey::SeasonalSnowFraction,
            FieldKey::MeltRate,
            FieldKey::FreezeRate,
            FieldKey::PlanetaryIceFraction,
            FieldKey::CryosphereAlbedoModifier,
            FieldKey::SeaLevelOffset,
        ]
    }

    fn shutdown(&mut self, _state: &mut dyn StateWriter) -> ContractResult<()> {
        Ok(())
    }

    fn publish_events(&mut self) -> Vec<SimulationEvent> {
        Vec::new()
    }

    fn consume_events(&mut self, _events: &[SimulationEvent]) -> ContractResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AtmosphereModule, BiosphereModule, CarbonCycleModule, ClimateModule, CoreEvolutionModule,
        HydrologyModule, MantleEvolutionModule, PlateTectonicsModule, VolcanismModule,
    };
    use worldsmith_engine::EngineBuilder;
    use worldsmith_math::Vector3;
    use worldsmith_models::{
        AtmosphericGas, AtmosphericProperties, BiosphereState, CarbonCycleState, MeasuredValue,
        OceanProperties, OrbitalProperties, PhysicalProperties, PlanetClassificationState, Star,
        StarId, SystemId,
    };

    fn earth_like_planet(planet_id: PlanetId, star_id: StarId) -> worldsmith_models::Planet {
        worldsmith_models::Planet {
            id: planet_id,
            name: format!("Planet {}", planet_id.0),
            class: worldsmith_models::PlanetClass::Terrestrial,
            planet_type: worldsmith_models::PlanetType::Rocky,
            system_id: SystemId(1),
            physical: PhysicalProperties {
                mass_kg: MeasuredValue {
                    value: 5.972e24,
                    unit: "kg".into(),
                    provenance: None,
                },
                radius_m: MeasuredValue {
                    value: 6.371e6,
                    unit: "m".into(),
                    provenance: None,
                },
                density_kg_m3: None,
                surface_gravity_m_s2: None,
            },
            orbit: OrbitalProperties {
                parent: worldsmith_models::BodyReference::Star(star_id),
                semi_major_axis_m: MeasuredValue {
                    value: 1.496e11,
                    unit: "m".into(),
                    provenance: None,
                },
                semi_minor_axis_m: None,
                eccentricity: MeasuredValue {
                    value: 0.0167,
                    unit: "dimensionless".into(),
                    provenance: None,
                },
                inclination_rad: MeasuredValue {
                    value: 0.0,
                    unit: "rad".into(),
                    provenance: None,
                },
                orbital_period_s: None,
                rotation_period_s: None,
                axial_tilt_rad: None,
            },
            geology: None,
            atmosphere: Some(AtmosphericProperties {
                atmosphere_type: worldsmith_models::AtmosphereType::None,
                pressure_pa: None,
                density_kg_m3: None,
                scale_height_m: None,
                layers: vec![],
                composition: vec![],
                cloud_coverage: None,
                greenhouse_gases: vec![],
            }),
            atmosphere_state: Some(AtmosphereState {
                atmospheric_mass_kg: 5.15e18,
                surface_pressure_pa: 101_325.0,
                mean_temperature_k: 288.0,
                atmosphere_composition: vec![AtmosphericGas {
                    molecule: worldsmith_models::Molecule {
                        formula: "CO2".into(),
                        name: "Carbon Dioxide".into(),
                        molar_mass_kg_mol: Some(MeasuredValue {
                            value: 0.04401,
                            unit: "kg/mol".into(),
                            provenance: None,
                        }),
                    },
                    abundance: MeasuredValue {
                        value: 0.0004,
                        unit: "dimensionless".into(),
                        provenance: None,
                    },
                    is_greenhouse: true,
                }],
            }),
            hydrology_state: Some(HydrologyState {
                total_water_mass_kg: 1.4e21,
                ocean_mass_kg: 1.4e21,
                atmospheric_water_mass_kg: 1.0e16,
                ice_mass_kg: 0.0,
                liquid_water_fraction: 1.0,
            }),
            climate_state: Some(ClimateState {
                equilibrium_temperature_k: 255.0,
                greenhouse_temperature_offset_k: 33.0,
                planetary_albedo: 0.3,
                climate_classification: worldsmith_models::ClimateType::Temperate,
            }),
            carbon_cycle_state: Some(CarbonCycleState::default()),
            biosphere_state: Some(BiosphereState::default()),
            habitability_state: Some(HabitabilityState::default()),
            classification_state: Some(PlanetClassificationState::default()),
            surface_chemistry_state: Some(SurfaceChemistryState::default()),
            cryosphere_state: Some(CryosphereState::default()),
            interior: None,
            volcanism: Some(worldsmith_models::VolcanismState {
                volcanic_flux: 1.0e10,
                volcanic_activity: worldsmith_models::VolcanicActivity::Moderate,
                magma_generation_rate: 1.0e10,
            }),
            plate_tectonics: Some(worldsmith_models::PlateTectonicsState {
                plate_velocity: 5.0,
                crustal_recycling_rate: 1.0e9,
                tectonic_activity: worldsmith_models::TectonicActivity::Moderate,
            }),
            climate: None,
            ocean: Some(OceanProperties {
                ocean_type: worldsmith_models::OceanType::Water,
                coverage: None,
                average_depth_m: None,
                composition: vec![],
            }),
            magnetic_field: None,
            habitability: None,
            position_m: Vector3::ZERO,
            velocity_m_s: Vector3::ZERO,
            moons: Vec::new(),
        }
    }

    fn default_star() -> Star {
        Star {
            id: StarId(1),
            name: "Sol".into(),
            spectral_type: worldsmith_models::SpectralType::G,
            class: worldsmith_models::StarClass::MainSequence,
            mass_kg: MeasuredValue {
                value: 1.989e30,
                unit: "kg".into(),
                provenance: None,
            },
            radius_m: MeasuredValue {
                value: 6.96e8,
                unit: "m".into(),
                provenance: None,
            },
            luminosity_w: MeasuredValue {
                value: 5.0e26,
                unit: "W".into(),
                provenance: None,
            },
            effective_temperature_k: MeasuredValue {
                value: 5778.0,
                unit: "K".into(),
                provenance: None,
            },
            surface_gravity_m_s2: MeasuredValue {
                value: 274.0,
                unit: "m/s2".into(),
                provenance: None,
            },
            metallicity: MeasuredValue {
                value: 0.0,
                unit: "dimensionless".into(),
                provenance: None,
            },
            rotation_period_s: None,
            age_s: Some(MeasuredValue {
                value: 4.6e17,
                unit: "s".into(),
                provenance: None,
            }),
            position_m: Vector3::ZERO,
            velocity_m_s: Vector3::ZERO,
        }
    }

    #[test]
    fn module_constructs_with_defaults() {
        let module = CryosphereModule::default();
        assert_eq!(module.id(), "worldsmith.evolution.cryosphere");
    }

    #[test]
    fn cryosphere_mutates_cryosphere_state() {
        let mut engine = EngineBuilder::new()
            .with_seed(7)
            .register_module(Box::new(CoreEvolutionModule::default()))
            .register_module(Box::new(MantleEvolutionModule::default()))
            .register_module(Box::new(VolcanismModule::default()))
            .register_module(Box::new(PlateTectonicsModule::default()))
            .register_module(Box::new(AtmosphereModule::default()))
            .register_module(Box::new(HydrologyModule::default()))
            .register_module(Box::new(ClimateModule::default()))
            .register_module(Box::new(CarbonCycleModule::default()))
            .register_module(Box::new(BiosphereModule::default()))
            .register_module(Box::new(CryosphereModule::default()))
            .build()
            .unwrap();

        let planet_id = PlanetId(1);
        let star_id = StarId(1);
        let planet = earth_like_planet(planet_id, star_id);

        engine.state_mut().planets.insert(planet_id, planet);
        engine.state_mut().stars.insert(star_id, default_star());
        let _ = engine.initialize();
        let _ = engine.tick(100.0);
        let _ = engine.tick(100.0);

        let updated = engine
            .state()
            .planets
            .get(&planet_id)
            .and_then(|p| p.cryosphere_state.clone())
            .unwrap();

        assert!(
            updated.continental_ice_mass_kg.is_finite(),
            "continental ice must be finite"
        );
        assert!(
            updated.sea_ice_mass_kg.is_finite(),
            "sea ice must be finite"
        );
        assert!(
            updated.snow_mass_kg >= 0.0,
            "snow mass must be non-negative"
        );
        assert!(
            updated.permanent_ice_fraction >= 0.0 && updated.permanent_ice_fraction <= 1.0,
            "permanent ice fraction must be in [0, 1]"
        );
        assert!(
            updated.seasonal_snow_fraction >= 0.0 && updated.seasonal_snow_fraction <= 1.0,
            "seasonal snow fraction must be in [0, 1]"
        );
        assert!(
            updated.planetary_ice_fraction >= 0.0 && updated.planetary_ice_fraction <= 1.0,
            "planetary ice fraction must be in [0, 1]"
        );
        assert!(
            updated.melt_rate_kg_per_s >= 0.0,
            "melt rate must be non-negative"
        );
        assert!(
            updated.freeze_rate_kg_per_s >= 0.0,
            "freeze rate must be non-negative"
        );
    }

    #[test]
    fn climate_drives_cryosphere_state() {
        let mut engine = EngineBuilder::new()
            .with_seed(7)
            .register_module(Box::new(CryosphereModule::default()))
            .build()
            .unwrap();

        let planet_id = PlanetId(1);
        let star_id = StarId(1);
        let mut planet = earth_like_planet(planet_id, star_id);
        planet.climate_state = Some(ClimateState {
            equilibrium_temperature_k: 260.0,
            greenhouse_temperature_offset_k: 0.0,
            planetary_albedo: 0.3,
            climate_classification: worldsmith_models::ClimateType::Cold,
        });

        engine.state_mut().planets.insert(planet_id, planet);
        engine.state_mut().stars.insert(star_id, default_star());
        let _ = engine.initialize();
        let _ = engine.tick(100.0);

        let updated = engine
            .state()
            .planets
            .get(&planet_id)
            .and_then(|p| p.cryosphere_state.clone())
            .unwrap();

        assert!(
            updated.freeze_rate_kg_per_s > 0.0 || updated.continental_ice_mass_kg > 0.0,
            "cold climate should produce freeze or ice mass"
        );
    }

    #[test]
    fn snapshots_contain_cryosphere_state() {
        let mut engine = EngineBuilder::new()
            .with_seed(7)
            .register_module(Box::new(CoreEvolutionModule::default()))
            .register_module(Box::new(MantleEvolutionModule::default()))
            .register_module(Box::new(VolcanismModule::default()))
            .register_module(Box::new(PlateTectonicsModule::default()))
            .register_module(Box::new(AtmosphereModule::default()))
            .register_module(Box::new(HydrologyModule::default()))
            .register_module(Box::new(ClimateModule::default()))
            .register_module(Box::new(CarbonCycleModule::default()))
            .register_module(Box::new(BiosphereModule::default()))
            .register_module(Box::new(CryosphereModule::default()))
            .build()
            .unwrap();

        let planet_id = PlanetId(1);
        let star_id = StarId(1);
        let planet = earth_like_planet(planet_id, star_id);

        engine.state_mut().planets.insert(planet_id, planet);
        engine.state_mut().stars.insert(star_id, default_star());
        let _ = engine.initialize();
        let _ = engine.tick(100.0);

        let snapshot = engine.latest_snapshot().expect("snapshot");
        let planet_snapshot = snapshot
            .planets
            .iter()
            .find(|p| p.id == planet_id)
            .expect("planet snapshot must exist");

        assert!(
            planet_snapshot.planet.cryosphere_state.is_some(),
            "snapshot must preserve cryosphere state"
        );
    }

    #[test]
    fn cryosphere_does_not_modify_climate_state() {
        let mut engine = EngineBuilder::new()
            .with_seed(7)
            .register_module(Box::new(CryosphereModule::default()))
            .build()
            .unwrap();

        let planet_id = PlanetId(1);
        let star_id = StarId(1);
        let planet = earth_like_planet(planet_id, star_id);
        let initial_climate = planet.climate_state.clone().unwrap();

        engine.state_mut().planets.insert(planet_id, planet);
        engine.state_mut().stars.insert(star_id, default_star());
        let _ = engine.initialize();
        let _ = engine.tick(100.0);

        let updated = engine
            .state()
            .planets
            .get(&planet_id)
            .and_then(|p| p.climate_state.clone())
            .unwrap();

        assert_eq!(
            updated.equilibrium_temperature_k, initial_climate.equilibrium_temperature_k,
            "CryosphereModule must not mutate ClimateState::equilibrium_temperature_k"
        );
        assert_eq!(
            updated.greenhouse_temperature_offset_k,
            initial_climate.greenhouse_temperature_offset_k,
            "CryosphereModule must not mutate ClimateState::greenhouse_temperature_offset_k"
        );
    }
}
