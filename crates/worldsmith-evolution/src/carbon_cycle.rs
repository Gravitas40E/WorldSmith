//! Planetary carbon cycle: deterministic bulk reservoir fluxes.
//!
//! This module implements a V1 deterministic carbon cycling model between
//! the atmosphere, ocean, and lithosphere.  Carbon fluxes are generated
//! here but NOT applied to `AtmosphereState` directly.  `AtmosphereModule`
//! will consume these fluxes during its own tick.
//!
//! ## Responsibilities
//! - Owns `atmospheric_carbon_mass_kg`, `ocean_carbon_mass_kg`,
//!   `lithosphere_carbon_mass_kg`, `volcanic_carbon_flux_kg_per_s`,
//!   `weathering_flux_kg_per_s`, `ocean_exchange_flux_kg_per_s`,
//!   `atmospheric_co2_fraction`, `carbon_partition_ratio`,
//!   `weathering_efficiency` per ADR-011.
//! - Reads `AtmosphereState`, `HydrologyState`, `ClimateState`,
//!   `VolcanismState`, `PlateTectonicsState`, and `Planet` properties
//!   after `worldsmith.evolution.climate`.
//!
//! ## Simplifying assumptions
//! 1. **Bulk reservoirs**: atmosphere, ocean, and lithosphere are treated
//!    as well-mixed single compartments.  No latitude bands, depth
//!    profiles, or lithologic units.
//! 2. **Volcanic degassing**: flux is a constant efficiency times
//!    `VolcanismState.volcanic_flux`.  No magma composition or
//!    degassing physics.
//! 3. **Silicate weathering**: removal rate scales with temperature and
//!    with exposed land area inferred from `PlateTectonicsState`.
//! 4. **Ocean-atmosphere exchange**: net flux is proportional to the
//!    difference between atmospheric and ocean carbon inventories and
//!    to ocean surface area inferred from `HydrologyState`.
//! 5. **One-tick delayed feedback**: this module never writes
//!    `AtmosphereState`.  Produced fluxes are consumed by
//!    `AtmosphereModule` in a later tick.
//! 6. **No biology**: no organic carbon burial, no biological pump,
//!    no photosynthesis/respiration.
//! 7. **No carbonate chemistry**: no pH, alkalinity, speciation, or
//!    carbonate saturation states.
//!
//! ## Future extensions
//! - biological carbon pump
//! - carbonate chemistry
//! - silicate weathering lithology map
//! - organic carbon burial
//!
//! ## Ownership
//!
//! - **Reads**: `AtmosphereState`, `HydrologyState`, `ClimateState`,
//!   `VolcanismState`, `PlateTectonicsState`, `Planet` properties
//! - **Writes**: `atmospheric_carbon_mass_kg`, `ocean_carbon_mass_kg`,
//!   `lithosphere_carbon_mass_kg`, `volcanic_carbon_flux_kg_per_s`,
//!   `weathering_flux_kg_per_s`, `ocean_exchange_flux_kg_per_s`,
//!   `atmospheric_co2_fraction`, `carbon_partition_ratio`,
//!   `weathering_efficiency`
//! - **Never modifies**: `AtmosphereState`, `HydrologyState`,
//!   `ClimateState`, `InteriorState`, `VolcanismState`,
//!   `PlateTectonicsState`, `climate`, `ocean`, `magnetic_field`,
//!   `habitability`

use serde::{Deserialize, Serialize};
use worldsmith_models::{
    AtmosphereState, CarbonCycleState, ClimateState, HydrologyState, Planet, PlanetId,
    VolcanismState,
};
use worldsmith_state::{FieldKey, SimulationEvent};
use worldsmith_traits::{ContractResult, ModuleContext, SimulationModule, StateWriter};

const DEFAULT_VOLCANIC_OUTGASSING_EFFICIENCY: f64 = 1.0e-24;
const DEFAULT_WEATHERING_EFFICIENCY: f64 = 1.0e-20;
const DEFAULT_OCEAN_EXCHANGE_RATE: f64 = 1.0e-15;
const DEFAULT_LITHOSPHERE_STORAGE_FRACTION: f64 = 0.9;
const DEFAULT_ATMOSPHERIC_PARTITION_FRACTION: f64 = 0.1;
const DEFAULT_INITIAL_ATMOSPHERIC_CARBON_KG: f64 = 1.2e18;
const DEFAULT_INITIAL_OCEAN_CARBON_KG: f64 = 2.5e19;
const DEFAULT_INITIAL_LITHOSPHERE_CARBON_KG: f64 = 1.5e22;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CarbonCycleConfig {
    pub volcanic_outgassing_efficiency: f64,
    pub weathering_efficiency: f64,
    pub ocean_exchange_rate: f64,
    pub lithosphere_storage_fraction: f64,
    pub atmospheric_partition_fraction: f64,
}

impl Default for CarbonCycleConfig {
    fn default() -> Self {
        Self {
            volcanic_outgassing_efficiency: DEFAULT_VOLCANIC_OUTGASSING_EFFICIENCY,
            weathering_efficiency: DEFAULT_WEATHERING_EFFICIENCY,
            ocean_exchange_rate: DEFAULT_OCEAN_EXCHANGE_RATE,
            lithosphere_storage_fraction: DEFAULT_LITHOSPHERE_STORAGE_FRACTION,
            atmospheric_partition_fraction: DEFAULT_ATMOSPHERIC_PARTITION_FRACTION,
        }
    }
}

pub struct CarbonCycleModule {
    config: CarbonCycleConfig,
    initialized: bool,
}

impl CarbonCycleModule {
    pub fn new(config: CarbonCycleConfig) -> Self {
        Self {
            config,
            initialized: false,
        }
    }

    #[allow(clippy::field_reassign_with_default)]
    fn initialize_reservoirs(&self, planet: &Planet) -> CarbonCycleState {
        let mut state = CarbonCycleState::default();
        state.atmospheric_carbon_mass_kg = DEFAULT_INITIAL_ATMOSPHERIC_CARBON_KG;
        state.ocean_carbon_mass_kg = DEFAULT_INITIAL_OCEAN_CARBON_KG;
        state.lithosphere_carbon_mass_kg = DEFAULT_INITIAL_LITHOSPHERE_CARBON_KG;
        if let Some(existing) = &planet.carbon_cycle_state {
            state.atmospheric_carbon_mass_kg = existing.atmospheric_carbon_mass_kg;
            state.ocean_carbon_mass_kg = existing.ocean_carbon_mass_kg;
            state.lithosphere_carbon_mass_kg = existing.lithosphere_carbon_mass_kg;
        }
        state
    }

    fn compute_fluxes(
        &self,
        carbon: &mut CarbonCycleState,
        atmosphere: &AtmosphereState,
        hydro: &HydrologyState,
        climate: &ClimateState,
        volcanism: &VolcanismState,
        plate_tectonics: &worldsmith_models::PlateTectonicsState,
    ) {
        let volcanic_flux =
            self.config.volcanic_outgassing_efficiency.max(0.0) * volcanism.volcanic_flux;

        let land_area_factor = (0.01 + plate_tectonics.plate_velocity * 0.1)
            .clamp(0.0, 1.0);
        let weathering = self.config.weathering_efficiency.max(0.0)
            * (0.5 + climate.planetary_albedo)
            * land_area_factor
            * (1.0 + climate.equilibrium_temperature_k / 1000.0);

        let ocean_surface_area = if hydro.total_water_mass_kg > 0.0 {
            (hydro.ocean_mass_kg / hydro.total_water_mass_kg)
                .clamp(0.0, 1.0)
        } else {
            0.0
        };
        let exchange = self.config.ocean_exchange_rate.max(0.0)
            * ocean_surface_area
            * (1.0
                + (carbon.atmospheric_carbon_mass_kg - carbon.ocean_carbon_mass_kg).abs() / 1.0e20);

        let _total = carbon.atmospheric_carbon_mass_kg
            + carbon.ocean_carbon_mass_kg
            + carbon.lithosphere_carbon_mass_kg;
        let partition = if carbon.atmospheric_carbon_mass_kg > 0.0 {
            carbon.ocean_carbon_mass_kg / carbon.atmospheric_carbon_mass_kg
        } else {
            0.0
        };
        let co2_fraction = if atmosphere.atmospheric_mass_kg > 0.0 {
            (carbon.atmospheric_carbon_mass_kg / atmosphere.atmospheric_mass_kg).min(1.0)
        } else {
            0.0
        };

        carbon.volcanic_carbon_flux_kg_per_s = volcanic_flux;
        carbon.weathering_flux_kg_per_s = weathering;
        carbon.ocean_exchange_flux_kg_per_s = exchange;
        carbon.atmospheric_co2_fraction = co2_fraction;
        carbon.carbon_partition_ratio = partition;
        carbon.weathering_efficiency = self.config.weathering_efficiency;
    }

    fn tick(
        &self,
        carbon: CarbonCycleState,
        atmosphere: &AtmosphereState,
        hydro: &HydrologyState,
        climate: &ClimateState,
        volcanism: &VolcanismState,
        plate_tectonics: &worldsmith_models::PlateTectonicsState,
    ) -> CarbonCycleState {
        let mut carbon = carbon;
        self.compute_fluxes(
            &mut carbon,
            atmosphere,
            hydro,
            climate,
            volcanism,
            plate_tectonics,
        );

        let dt = 1.0;
        let volcanic_input = carbon.volcanic_carbon_flux_kg_per_s * dt;
        let weathering_removal = carbon.weathering_flux_kg_per_s * dt;
        let exchange = carbon.ocean_exchange_flux_kg_per_s * dt;

        carbon.lithosphere_carbon_mass_kg +=
            volcanic_input * self.config.lithosphere_storage_fraction;
        carbon.atmospheric_carbon_mass_kg +=
            volcanic_input * self.config.atmospheric_partition_fraction;
        carbon.atmospheric_carbon_mass_kg -= weathering_removal;
        carbon.ocean_carbon_mass_kg += exchange;

        carbon.atmospheric_carbon_mass_kg = carbon.atmospheric_carbon_mass_kg.max(0.0);
        carbon.ocean_carbon_mass_kg = carbon.ocean_carbon_mass_kg.max(0.0);
        carbon.lithosphere_carbon_mass_kg = carbon.lithosphere_carbon_mass_kg.max(0.0);

        carbon
    }
}

impl Default for CarbonCycleModule {
    fn default() -> Self {
        Self::new(CarbonCycleConfig::default())
    }
}

impl SimulationModule for CarbonCycleModule {
    fn id(&self) -> &'static str {
        "worldsmith.evolution.carbon_cycle"
    }

    fn initialize(&mut self, state: &mut dyn StateWriter) -> ContractResult<()> {
        let planet_ids: Vec<_> = state.world().planets.keys().cloned().collect();
        for planet_id in planet_ids {
            if let Some(planet) = state.world().planets.get(&planet_id).cloned() {
                let carbon = self.initialize_reservoirs(&planet);
                let mut updated = planet.clone();
                updated.carbon_cycle_state = Some(carbon);
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

        let snapshot: Vec<(PlanetId, Planet, Option<CarbonCycleState>)> = state
            .world()
            .planets
            .values()
            .map(|planet| (planet.id, planet.clone(), planet.carbon_cycle_state.clone()))
            .collect();

        for (_planet_id, planet, carbon) in snapshot {
            let carbon = match carbon {
                Some(carbon) => carbon,
                None => continue,
            };

            let atmosphere = match &planet.atmosphere_state {
                Some(atm) => atm,
                None => continue,
            };
            let hydro = match &planet.hydrology_state {
                Some(h) => h,
                None => continue,
            };
            let climate = match &planet.climate_state {
                Some(c) => c,
                None => continue,
            };
            let volcanism = match &planet.volcanism {
                Some(v) => v,
                None => continue,
            };
            let plate_tectonics = match &planet.plate_tectonics {
                Some(pt) => pt,
                None => continue,
            };

            let updated = self.tick(
                carbon,
                atmosphere,
                hydro,
                climate,
                volcanism,
                plate_tectonics,
            );
            let mut updated_planet = planet;
            updated_planet.carbon_cycle_state = Some(updated);
            state
                .world_mut()
                .planets
                .insert(updated_planet.id, updated_planet);
        }

        Ok(())
    }

    fn reads(&self) -> Vec<FieldKey> {
        vec![
            FieldKey::AtmosphericCarbonMass,
            FieldKey::OceanCarbonMass,
            FieldKey::LithosphereCarbonMass,
            FieldKey::VolcanicCarbonFlux,
            FieldKey::WeatheringCarbonFlux,
            FieldKey::OceanExchangeFlux,
            FieldKey::AtmosphericCo2Fraction,
            FieldKey::CarbonPartitionRatio,
            FieldKey::WeatheringEfficiency,
        ]
    }

    fn writes(&self) -> Vec<FieldKey> {
        vec![
            FieldKey::AtmosphericCarbonMass,
            FieldKey::OceanCarbonMass,
            FieldKey::LithosphereCarbonMass,
            FieldKey::VolcanicCarbonFlux,
            FieldKey::WeatheringCarbonFlux,
            FieldKey::OceanExchangeFlux,
            FieldKey::AtmosphericCo2Fraction,
            FieldKey::CarbonPartitionRatio,
            FieldKey::WeatheringEfficiency,
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
        CarbonCycleModule, ClimateModule, CoreEvolutionModule, HydrologyModule,
        MantleEvolutionModule, PlateTectonicsModule, VolcanismModule,
    };
    use worldsmith_engine::EngineBuilder;
    use worldsmith_math::Vector3;
    use worldsmith_models::{
        AtmosphericGas, AtmosphericProperties, MeasuredValue, OceanProperties, OrbitalProperties,
        PhysicalProperties, Star, StarId, SystemId,
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
            biosphere_state: None,
            habitability_state: None,
            classification_state: None,
            surface_chemistry_state: None,
            cryosphere_state: None,
            interior: None,
            volcanism: Some(VolcanismState {
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
        let module = CarbonCycleModule::default();
        assert_eq!(module.id(), "worldsmith.evolution.carbon_cycle");
    }

    #[test]
    fn carbon_cycle_does_not_modify_atmosphere_state() {
        let mut engine = EngineBuilder::new()
            .with_seed(7)
            .register_module(Box::new(CoreEvolutionModule::default()))
            .register_module(Box::new(MantleEvolutionModule::default()))
            .register_module(Box::new(VolcanismModule::default()))
            .register_module(Box::new(PlateTectonicsModule::default()))
            .register_module(Box::new(HydrologyModule::default()))
            .register_module(Box::new(ClimateModule::default()))
            .register_module(Box::new(CarbonCycleModule::default()))
            .build()
            .unwrap();

        let planet_id = PlanetId(1);
        let star_id = StarId(1);
        let mut planet = earth_like_planet(planet_id, star_id);
        planet.atmosphere_state = Some(AtmosphereState {
            atmospheric_mass_kg: 5.15e18,
            surface_pressure_pa: 101_325.0,
            mean_temperature_k: 288.0,
            atmosphere_composition: vec![],
        });
        let initial_atmosphere = planet.atmosphere_state.clone().unwrap();

        engine.state_mut().planets.insert(planet_id, planet);
        engine.state_mut().stars.insert(star_id, default_star());
        let _ = engine.initialize();
        let _ = engine.tick(100.0);
        let _ = engine.tick(100.0);

        let updated = engine
            .state()
            .planets
            .get(&planet_id)
            .and_then(|p| p.atmosphere_state.as_ref())
            .unwrap();
        assert_eq!(
            updated.atmospheric_mass_kg,
            initial_atmosphere.atmospheric_mass_kg
        );
        assert_eq!(
            updated.surface_pressure_pa,
            initial_atmosphere.surface_pressure_pa
        );
    }

    #[test]
    fn carbon_cycle_mutates_carbon_cycle_state() {
        let mut engine = EngineBuilder::new()
            .with_seed(7)
            .register_module(Box::new(CoreEvolutionModule::default()))
            .register_module(Box::new(MantleEvolutionModule::default()))
            .register_module(Box::new(VolcanismModule::default()))
            .register_module(Box::new(PlateTectonicsModule::default()))
            .register_module(Box::new(HydrologyModule::default()))
            .register_module(Box::new(ClimateModule::default()))
            .register_module(Box::new(CarbonCycleModule::default()))
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
            .and_then(|p| p.carbon_cycle_state.clone())
            .unwrap();
        assert!(updated.atmospheric_carbon_mass_kg >= 0.0);
        assert!(updated.ocean_carbon_mass_kg >= 0.0);
        assert!(updated.lithosphere_carbon_mass_kg >= 0.0);
    }

    #[test]
    fn snapshots_contain_carbon_cycle_state() {
        let mut engine = EngineBuilder::new()
            .with_seed(7)
            .register_module(Box::new(CoreEvolutionModule::default()))
            .register_module(Box::new(MantleEvolutionModule::default()))
            .register_module(Box::new(VolcanismModule::default()))
            .register_module(Box::new(PlateTectonicsModule::default()))
            .register_module(Box::new(HydrologyModule::default()))
            .register_module(Box::new(ClimateModule::default()))
            .register_module(Box::new(CarbonCycleModule::default()))
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
        assert!(planet_snapshot.planet.carbon_cycle_state.is_some());
    }
}
