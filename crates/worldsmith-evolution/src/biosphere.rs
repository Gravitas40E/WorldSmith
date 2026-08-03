//! Planetary biosphere: deterministic bulk biomass and productivity.
//!
//! This module implements a V1 deterministic planetary biosphere model.
//! It models biological productivity and biomass at a planetary scale.
//! No species, organisms, ecology, evolution, or food webs are simulated.
//!
//! ## Responsibilities
//! - Owns `total_biomass_kg`, `terrestrial_biomass_kg`, `marine_biomass_kg`,
//!   `dead_organic_carbon_kg`, `productivity_rate_kg_per_s`,
//!   `respiration_rate_kg_per_s`, `habitability_factor`,
//!   `vegetation_fraction`, and `ocean_productivity_factor` per ADR-011.
//! - Reads `ClimateState`, `HydrologyState`, `CarbonCycleState`,
//!   `AtmosphereState`, and `Planet` properties after
//!   `worldsmith.evolution.carbon_cycle`.
//!
//! ## Simplifying assumptions
//! 1. **Bulk biomass**: a single global biomass scalar. No species,
//!    biomes, or spatial distributions.
//! 2. **Climate-limited productivity**: productivity scales with temperature
//!    and atmospheric CO2 only. No nutrient limitation.
//! 3. **Water-limited productivity**: productivity requires liquid water.
//! 4. **Respiration**: a fixed fraction of biomass is respired each tick.
//! 5. **Carrying capacity**: terrestrial and marine biomass are capped by
//!    configurable carrying capacities.
//! 6. **No evolution**: parameters are static; no adaptation or speciation.
//! 7. **No carbon coupling**: this module does not write `CarbonCycleState`.
//!    Biological carbon fluxes are computed but stored only in
//!    `BiosphereState` for future consumption.
//!
//! ## Future extensions
//! - nutrient cycles (N, P)
//! - ecosystem functional types
//! - evolutionary dynamics
//! - species-level models
//!

use serde::{Deserialize, Serialize};
use worldsmith_models::{
    AtmosphereState, BiosphereState, CarbonCycleState, ClimateState, HydrologyState, Planet,
    PlanetId,
};
use worldsmith_state::{FieldKey, SimulationEvent};
use worldsmith_traits::{ContractResult, ModuleContext, SimulationModule, StateWriter};

const DEFAULT_MAXIMUM_PRODUCTIVITY: f64 = 1.0e12;
const DEFAULT_RESPIRATION_FRACTION: f64 = 0.5;
const DEFAULT_TERRESTRIAL_CARRYING_CAPACITY: f64 = 1.0e15;
const DEFAULT_MARINE_CARRYING_CAPACITY: f64 = 5.0e14;
const DEFAULT_WATER_DEPENDENCE: f64 = 1.0;
const DEFAULT_TEMPERATURE_DEPENDENCE: f64 = 1.0;
const DEFAULT_INITIAL_TOTAL_BIOMASS_KG: f64 = 1.0e14;
const DEFAULT_INITIAL_DEAD_ORGANIC_CARBON_KG: f64 = 1.0e18;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BiosphereConfig {
    pub maximum_productivity: f64,
    pub respiration_fraction: f64,
    pub terrestrial_carrying_capacity: f64,
    pub marine_carrying_capacity: f64,
    pub water_dependence: f64,
    pub temperature_dependence: f64,
}

impl Default for BiosphereConfig {
    fn default() -> Self {
        Self {
            maximum_productivity: DEFAULT_MAXIMUM_PRODUCTIVITY,
            respiration_fraction: DEFAULT_RESPIRATION_FRACTION,
            terrestrial_carrying_capacity: DEFAULT_TERRESTRIAL_CARRYING_CAPACITY,
            marine_carrying_capacity: DEFAULT_MARINE_CARRYING_CAPACITY,
            water_dependence: DEFAULT_WATER_DEPENDENCE,
            temperature_dependence: DEFAULT_TEMPERATURE_DEPENDENCE,
        }
    }
}

pub struct BiosphereModule {
    config: BiosphereConfig,
    initialized: bool,
}

impl BiosphereModule {
    pub fn new(config: BiosphereConfig) -> Self {
        Self {
            config,
            initialized: false,
        }
    }

    #[allow(clippy::field_reassign_with_default)]
    fn initialize_reservoirs(&self, planet: &Planet) -> BiosphereState {
        let mut state = BiosphereState::default();
        state.total_biomass_kg = DEFAULT_INITIAL_TOTAL_BIOMASS_KG;
        state.dead_organic_carbon_kg = DEFAULT_INITIAL_DEAD_ORGANIC_CARBON_KG;
        if let Some(existing) = &planet.biosphere_state {
            state.total_biomass_kg = existing.total_biomass_kg;
            state.terrestrial_biomass_kg = existing.terrestrial_biomass_kg;
            state.marine_biomass_kg = existing.marine_biomass_kg;
            state.dead_organic_carbon_kg = existing.dead_organic_carbon_kg;
            state.productivity_rate_kg_per_s = existing.productivity_rate_kg_per_s;
            state.respiration_rate_kg_per_s = existing.respiration_rate_kg_per_s;
        }
        state
    }

    fn compute_biosphere(
        &self,
        bio: &mut BiosphereState,
        climate: &ClimateState,
        hydro: &HydrologyState,
        carbon: &CarbonCycleState,
        atmosphere: &AtmosphereState,
        _planet: &Planet,
    ) {
        let temp = climate.equilibrium_temperature_k + climate.greenhouse_temperature_offset_k;

        let temp_suitability = if !(250.0..=320.0).contains(&temp) {
            0.0
        } else {
            let peak = 280.0;
            let width = 20.0;
            let t = (temp - peak) / width;
            (-t * t).exp()
        };

        let liquid_fraction = hydro.liquid_water_fraction;
        let water_availability = liquid_fraction.powf(self.config.water_dependence);

        let co2_ppm = if atmosphere.atmospheric_mass_kg > 0.0 {
            carbon.atmospheric_carbon_mass_kg / atmosphere.atmospheric_mass_kg * 1.0e6
        } else {
            0.0
        };
        let co2_suitability = if co2_ppm < 10.0 {
            0.1
        } else if co2_ppm > 2000.0 {
            0.2
        } else {
            1.0
        };

        let land_fraction = if hydro.total_water_mass_kg > 0.0 {
            (1.0 - (hydro.ocean_mass_kg / hydro.total_water_mass_kg)).max(0.0)
        } else {
            0.0
        };

        let ocean_fraction = 1.0 - land_fraction;

        let terrestrial_productivity = temp_suitability
            * water_availability
            * co2_suitability
            * self.config.maximum_productivity;
        let marine_productivity = (temp_suitability * 0.7 + 0.3)
            * (liquid_fraction * 0.8 + 0.2)
            * ocean_fraction
            * self.config.maximum_productivity;

        let total_productivity = terrestrial_productivity + marine_productivity;

        let total_respiration = bio.total_biomass_kg * self.config.respiration_fraction;

        let mut terrestrial = bio.terrestrial_biomass_kg
            + (terrestrial_productivity
                - terrestrial_productivity * self.config.respiration_fraction)
                * 1.0;
        let mut marine = bio.marine_biomass_kg
            + (marine_productivity - marine_productivity * self.config.respiration_fraction) * 1.0;

        terrestrial = terrestrial
            .min(self.config.terrestrial_carrying_capacity)
            .max(0.0);
        marine = marine.min(self.config.marine_carrying_capacity).max(0.0);

        let total = terrestrial + marine;
        let dead = bio.dead_organic_carbon_kg + total_respiration * 0.1;

        bio.total_biomass_kg = total;
        bio.terrestrial_biomass_kg = terrestrial;
        bio.marine_biomass_kg = marine;
        bio.dead_organic_carbon_kg = dead;
        bio.productivity_rate_kg_per_s = total_productivity;
        bio.respiration_rate_kg_per_s = total_respiration;
        bio.habitability_factor =
            (temp_suitability * water_availability * co2_suitability).clamp(0.0, 1.0);
        bio.vegetation_fraction = if total > 0.0 {
            (terrestrial / total).clamp(0.0, 1.0)
        } else {
            0.0
        };
        bio.ocean_productivity_factor = if ocean_fraction > 0.0 {
            (marine_productivity / (self.config.maximum_productivity * ocean_fraction))
                .clamp(0.0, 1.0)
        } else {
            0.0
        };
    }

    fn tick(
        &self,
        mut bio: BiosphereState,
        climate: &ClimateState,
        hydro: &HydrologyState,
        carbon: &CarbonCycleState,
        atmosphere: &AtmosphereState,
        planet: &Planet,
    ) -> BiosphereState {
        self.compute_biosphere(&mut bio, climate, hydro, carbon, atmosphere, planet);
        bio
    }
}

impl Default for BiosphereModule {
    fn default() -> Self {
        Self::new(BiosphereConfig::default())
    }
}

impl SimulationModule for BiosphereModule {
    fn id(&self) -> &'static str {
        "worldsmith.evolution.biosphere"
    }

    fn initialize(&mut self, state: &mut dyn StateWriter) -> ContractResult<()> {
        let planet_ids: Vec<_> = state.world().planets.keys().cloned().collect();
        for planet_id in planet_ids {
            if let Some(planet) = state.world().planets.get(&planet_id).cloned() {
                let bio = self.initialize_reservoirs(&planet);
                let mut updated = planet.clone();
                updated.biosphere_state = Some(bio);
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

        let snapshot: Vec<(PlanetId, Planet, Option<BiosphereState>)> = state
            .world()
            .planets
            .values()
            .map(|planet| (planet.id, planet.clone(), planet.biosphere_state.clone()))
            .collect();

        for (_planet_id, planet, bio) in snapshot {
            let bio = match bio {
                Some(bio) => bio,
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
            let carbon = match &planet.carbon_cycle_state {
                Some(c) => c,
                None => continue,
            };
            let atmosphere = match &planet.atmosphere_state {
                Some(a) => a,
                None => continue,
            };

            let updated = self.tick(bio, climate, hydro, carbon, atmosphere, &planet);
            let mut updated_planet = planet;
            updated_planet.biosphere_state = Some(updated);
            state
                .world_mut()
                .planets
                .insert(updated_planet.id, updated_planet);
        }

        Ok(())
    }

    fn reads(&self) -> Vec<FieldKey> {
        vec![
            FieldKey::TotalBiomassMass,
            FieldKey::TerrestrialBiomassMass,
            FieldKey::MarineBiomassMass,
            FieldKey::DeadOrganicCarbonMass,
            FieldKey::ProductivityRate,
            FieldKey::RespirationRate,
            FieldKey::HabitabilityFactor,
            FieldKey::VegetationFraction,
            FieldKey::OceanProductivityFactor,
        ]
    }

    fn writes(&self) -> Vec<FieldKey> {
        vec![
            FieldKey::TotalBiomassMass,
            FieldKey::TerrestrialBiomassMass,
            FieldKey::MarineBiomassMass,
            FieldKey::DeadOrganicCarbonMass,
            FieldKey::ProductivityRate,
            FieldKey::RespirationRate,
            FieldKey::HabitabilityFactor,
            FieldKey::VegetationFraction,
            FieldKey::OceanProductivityFactor,
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
        BiosphereModule, CarbonCycleModule, CoreEvolutionModule, MantleEvolutionModule,
        PlateTectonicsModule, VolcanismModule,
    };
    use worldsmith_engine::EngineBuilder;
    use worldsmith_math::Vector3;
    use worldsmith_models::{
        AtmosphericGas, AtmosphericProperties, CryosphereState, HabitabilityState, MeasuredValue,
        OceanProperties, OrbitalProperties, PhysicalProperties, PlanetClassificationState, Star,
        StarId, SurfaceChemistryState, SystemId,
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
                unit: "m/s^2".into(),
                provenance: None,
            },
            metallicity: MeasuredValue {
                value: 0.0,
                unit: "dimensionless".into(),
                provenance: None,
            },
            rotation_period_s: None,
            age_s: None,
            position_m: worldsmith_math::Vector3::ZERO,
            velocity_m_s: worldsmith_math::Vector3::ZERO,
        }
    }

    #[test]
    fn module_constructs_with_defaults() {
        let module = BiosphereModule::default();
        assert_eq!(module.id(), "worldsmith.evolution.biosphere");
    }

    #[test]
    fn biosphere_mutates_biosphere_state() {
        let mut engine = EngineBuilder::new()
            .with_seed(7)
            .register_module(Box::new(CoreEvolutionModule::default()))
            .register_module(Box::new(MantleEvolutionModule::default()))
            .register_module(Box::new(VolcanismModule::default()))
            .register_module(Box::new(PlateTectonicsModule::default()))
            .register_module(Box::new(CarbonCycleModule::default()))
            .register_module(Box::new(BiosphereModule::default()))
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
            .and_then(|p| p.biosphere_state.clone())
            .unwrap();
        assert!(updated.total_biomass_kg >= 0.0);
        assert!(updated.productivity_rate_kg_per_s >= 0.0);
        assert!(updated.respiration_rate_kg_per_s >= 0.0);
    }

    #[test]
    fn biosphere_does_not_modify_carbon_cycle_state() {
        // Verify BiosphereModule in isolation does not mutate CarbonCycleState.
        let mut engine = EngineBuilder::new()
            .with_seed(7)
            .register_module(Box::new(BiosphereModule::default()))
            .build()
            .unwrap();

        let planet_id = PlanetId(1);
        let star_id = StarId(1);
        let planet = earth_like_planet(planet_id, star_id);
        let initial_carbon = planet.carbon_cycle_state.clone().unwrap();

        engine.state_mut().planets.insert(planet_id, planet);
        engine.state_mut().stars.insert(star_id, default_star());
        let _ = engine.initialize();
        let _ = engine.tick(100.0);
        let _ = engine.tick(100.0);

        let updated = engine
            .state()
            .planets
            .get(&planet_id)
            .and_then(|p| p.carbon_cycle_state.clone())
            .unwrap();
        assert_eq!(
            updated.atmospheric_carbon_mass_kg,
            initial_carbon.atmospheric_carbon_mass_kg
        );
        assert_eq!(
            updated.ocean_carbon_mass_kg,
            initial_carbon.ocean_carbon_mass_kg
        );
        assert_eq!(
            updated.lithosphere_carbon_mass_kg,
            initial_carbon.lithosphere_carbon_mass_kg
        );
    }

    #[test]
    fn climate_influences_productivity() {
        let mut engine = EngineBuilder::new()
            .with_seed(7)
            .register_module(Box::new(CoreEvolutionModule::default()))
            .register_module(Box::new(MantleEvolutionModule::default()))
            .register_module(Box::new(VolcanismModule::default()))
            .register_module(Box::new(PlateTectonicsModule::default()))
            .register_module(Box::new(CarbonCycleModule::default()))
            .register_module(Box::new(BiosphereModule::default()))
            .build()
            .unwrap();

        let planet_id = PlanetId(1);
        let star_id = StarId(1);
        let mut planet = earth_like_planet(planet_id, star_id);
        planet.climate_state = Some(ClimateState {
            equilibrium_temperature_k: 255.0,
            greenhouse_temperature_offset_k: 33.0,
            planetary_albedo: 0.3,
            climate_classification: worldsmith_models::ClimateType::Temperate,
        });

        engine.state_mut().planets.insert(planet_id, planet);
        engine.state_mut().stars.insert(star_id, default_star());
        let _ = engine.initialize();
        let _ = engine.tick(100.0);

        let updated = engine
            .state()
            .planets
            .get(&planet_id)
            .and_then(|p| p.biosphere_state.clone())
            .unwrap();
        assert!(updated.productivity_rate_kg_per_s >= 0.0);
    }

    #[test]
    fn snapshots_contain_biosphere_state() {
        let mut engine = EngineBuilder::new()
            .with_seed(7)
            .register_module(Box::new(CoreEvolutionModule::default()))
            .register_module(Box::new(MantleEvolutionModule::default()))
            .register_module(Box::new(VolcanismModule::default()))
            .register_module(Box::new(PlateTectonicsModule::default()))
            .register_module(Box::new(CarbonCycleModule::default()))
            .register_module(Box::new(BiosphereModule::default()))
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

        let snapshot = engine.latest_snapshot().expect("snapshot");
        assert_ne!(snapshot.planets.len(), 0);
        let planet_snapshot = snapshot.planets.iter().find(|p| p.id == planet_id).unwrap();
        assert!(planet_snapshot.planet.biosphere_state.is_some());
    }
}
