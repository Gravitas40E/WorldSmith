//! Planetary climate: deterministic global energy balance.
//!
//! This module implements a zero-dimensional planetary energy balance model.
//! Phase 11C introduces a V1 baseline with no weather, clouds, latitude bands,
//! circulation, seasons, or general circulation model.
//!
//! ## Responsibilities
//! - Owns `equilibrium_temperature_k`, `greenhouse_temperature_offset_k`,
//!   `planetary_albedo`, and `climate_classification` per ADR-011.
//! - Reads `AtmosphereState`, `HydrologyState`, `Planet.physical.radius_m`,
//!   and stellar luminosity after `worldsmith.evolution.hydrology`.
//!
//! ## Simplifying assumptions
//! 1. **Zero-dimensional energy balance**: the planet is a single
//!    blackbody sphere.  No spatial gradients, seasons, or diurnal cycles.
//! 2. **Bond albedo**: a single scalar albedo is computed from ice cover
//!    and ocean fraction; no cloud model in V1.
//! 3. **Linear greenhouse**: warming is a linear function of atmospheric
//!    mass above a baseline; no spectroscopy or optical-depth integration.
//! 4. **Deterministic classification**: thresholds are fixed config values;
//!    no fuzzy boundaries in V1.
//! 5. **No feedbacks**: climate does not evolve the atmosphere or hydrosphere
//!    and is not fed back into them.
//!
//! ## Future replacement
//!
//! This implementation is a deterministic zero-dimensional energy balance
//! model.  Future phases should introduce:
//!
//! - latitude-resolved energy balance
//! - general circulation model (GCM)
//! - cloud physics and radiative transfer
//! - regional climate response
//!
//! ## Ownership
//!
//! - **Reads**: `AtmosphereState`, `HydrologyState`,
//!   `Planet.physical.radius_m`, `Star.luminosity_w`
//! - **Writes**: `equilibrium_temperature_k`, `greenhouse_temperature_offset_k`,
//!   `planetary_albedo`, `climate_classification`
//! - **Never modifies**: `AtmosphereState`, `HydrologyState`, `InteriorState`,
//!   `VolcanismState`, `PlateTectonicsState`, `climate`, `ocean`,
//!   `magnetic_field`, `habitability`

use serde::{Deserialize, Serialize};
use worldsmith_models::{ClimateState, ClimateType, Planet, PlanetId};
use worldsmith_state::{FieldKey, SimulationEvent};
use worldsmith_traits::{ContractResult, ModuleContext, SimulationModule, StateWriter};

const STEFAN_BOLTZMANN: f64 = 5.670374419e-8;
const DEFAULT_CLOUDLESS_ALBEDO: f64 = 0.12;
const DEFAULT_GREENHOUSE_SCALING: f64 = 15.0;
const DEFAULT_EMISSIVITY: f64 = 1.0;
const DEFAULT_SOLAR_CONSTANT_REFERENCE: f64 = 1_361.0;
const DEFAULT_FROZEN_THRESHOLD: f64 = 250.0;
const DEFAULT_TEMPERATE_THRESHOLD: f64 = 285.0;
const DEFAULT_WARM_THRESHOLD: f64 = 320.0;
const DEFAULT_HOT_THRESHOLD: f64 = 400.0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClimateConfig {
    pub default_albedo: f64,
    pub cloudless_albedo: f64,
    pub greenhouse_scaling: f64,
    pub emissivity: f64,
    pub solar_constant_reference: f64,
    pub frozen_threshold: f64,
    pub temperate_threshold: f64,
    pub warm_threshold: f64,
    pub hot_threshold: f64,
}

impl Default for ClimateConfig {
    fn default() -> Self {
        Self {
            default_albedo: DEFAULT_CLOUDLESS_ALBEDO,
            cloudless_albedo: DEFAULT_CLOUDLESS_ALBEDO,
            greenhouse_scaling: DEFAULT_GREENHOUSE_SCALING,
            emissivity: DEFAULT_EMISSIVITY,
            solar_constant_reference: DEFAULT_SOLAR_CONSTANT_REFERENCE,
            frozen_threshold: DEFAULT_FROZEN_THRESHOLD,
            temperate_threshold: DEFAULT_TEMPERATE_THRESHOLD,
            warm_threshold: DEFAULT_WARM_THRESHOLD,
            hot_threshold: DEFAULT_HOT_THRESHOLD,
        }
    }
}

pub struct ClimateModule {
    config: ClimateConfig,
    initialized: bool,
}

impl ClimateModule {
    pub fn new(config: ClimateConfig) -> Self {
        Self {
            config,
            initialized: false,
        }
    }

    fn apply_initial_climate(&self, planet: &Planet) -> ContractResult<Planet> {
        let mut updated = planet.clone();
        if updated.climate_state.is_none() {
            updated.climate_state = Some(ClimateState::default());
        }
        Ok(updated)
    }

    fn classify(&self, effective_temperature: f64) -> ClimateType {
        if effective_temperature < self.config.frozen_threshold {
            ClimateType::Frozen
        } else if effective_temperature < self.config.temperate_threshold {
            ClimateType::Temperate
        } else if effective_temperature < self.config.warm_threshold {
            ClimateType::Tropical
        } else {
            ClimateType::RunawayGreenhouse
        }
    }

    fn compute_climate(
        &self,
        planet: &Planet,
        _dt: f64,
        luminosity: f64,
        semi_major_axis: f64,
    ) -> ContractResult<ClimateState> {
        let atmosphere = planet.atmosphere_state.clone().unwrap_or_default();

        let hydro = planet.hydrology_state.clone().unwrap_or_default();

        let total_water = hydro.total_water_mass_kg;
        let ocean_mass = hydro.ocean_mass_kg;
        let ice_mass = hydro.ice_mass_kg;

        let ice_fraction = if total_water > 0.0 {
            (ice_mass / total_water).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let ocean_fraction = if total_water > 0.0 {
            (ocean_mass / total_water).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let mut albedo = self.config.cloudless_albedo;

        if ice_fraction > 0.0 {
            let ice_albedo = 0.6;
            albedo = albedo * (1.0 - ice_fraction) + ice_albedo * ice_fraction;
        }

        if ocean_fraction > 0.0 {
            let ocean_albedo = 0.06;
            albedo = albedo * (1.0 - ocean_fraction) + ocean_albedo * ocean_fraction;
        }

        albedo = albedo.clamp(0.0, 1.0);

        let t_eq = if luminosity > 0.0 && semi_major_axis > 0.0 {
            ((luminosity * (1.0 - albedo))
                / (16.0
                    * std::f64::consts::PI
                    * STEFAN_BOLTZMANN
                    * semi_major_axis
                    * semi_major_axis))
                .max(0.0)
                .powf(0.25)
        } else {
            0.0
        };

        let co2_fraction = atmosphere
            .atmosphere_composition
            .iter()
            .find(|g| g.molecule.formula == "CO2")
            .map(|g| g.abundance.value)
            .unwrap_or(0.0);

        let atm_mass_kg = atmosphere.atmospheric_mass_kg;
        let greenhouse_offset =
            self.config.greenhouse_scaling * (atm_mass_kg / 1.0e18).max(0.0) * (0.5 + co2_fraction);

        let greenhouse_offset = greenhouse_offset.clamp(0.0, t_eq);

        let effective_temperature = t_eq + greenhouse_offset;

        let classification = self.classify(effective_temperature);

        Ok(ClimateState {
            equilibrium_temperature_k: t_eq,
            greenhouse_temperature_offset_k: greenhouse_offset,
            planetary_albedo: albedo,
            climate_classification: classification,
        })
    }
}

impl Default for ClimateModule {
    fn default() -> Self {
        Self::new(ClimateConfig::default())
    }
}

impl SimulationModule for ClimateModule {
    fn id(&self) -> &'static str {
        "worldsmith.evolution.climate"
    }

    fn initialize(&mut self, state: &mut dyn StateWriter) -> ContractResult<()> {
        let planet_ids: Vec<_> = state.world().planets.keys().cloned().collect();
        for planet_id in planet_ids {
            if let Some(planet) = state.world().planets.get(&planet_id).cloned() {
                let mut updated = self.apply_initial_climate(&planet)?;
                if updated.climate_state.is_some() {
                    let (luminosity, semi_major_axis) = match planet.orbit.parent {
                        worldsmith_models::BodyReference::Star(star_id) => {
                            let star = state.world().stars.get(&star_id);
                            match star {
                                Some(s) => {
                                    (s.luminosity_w.value, planet.orbit.semi_major_axis_m.value)
                                }
                                None => (0.0, 1.0),
                            }
                        }
                        _ => (0.0, 1.0),
                    };
                    updated.climate_state =
                        Some(self.compute_climate(&planet, 0.0, luminosity, semi_major_axis)?);
                }
                state.world_mut().planets.insert(updated.id, updated);
            }
        }
        self.initialized = true;
        Ok(())
    }

    fn update(
        &mut self,
        context: ModuleContext,
        state: &mut dyn StateWriter,
    ) -> ContractResult<()> {
        if !self.initialized {
            return Ok(());
        }

        let snapshot: Vec<(PlanetId, Planet, Option<ClimateState>)> = state
            .world()
            .planets
            .values()
            .map(|planet| (planet.id, planet.clone(), planet.climate_state.clone()))
            .collect();

        for (planet_id, planet, climate_state) in snapshot {
            let (luminosity, semi_major_axis) = match planet.orbit.parent {
                worldsmith_models::BodyReference::Star(star_id) => {
                    let star = state.world().stars.get(&star_id);
                    match star {
                        Some(s) => (s.luminosity_w.value, planet.orbit.semi_major_axis_m.value),
                        None => (0.0, 1.0),
                    }
                }
                _ => (0.0, 1.0),
            };

            let updated =
                self.compute_climate(&planet, context.delta_seconds, luminosity, semi_major_axis)?;

            if let Some(existing) = climate_state {
                if !existing.equilibrium_temperature_k.is_finite()
                    || !updated.equilibrium_temperature_k.is_finite()
                    || !updated.greenhouse_temperature_offset_k.is_finite()
                    || !updated.planetary_albedo.is_finite()
                {
                    return Err(worldsmith_traits::ContractError::ModuleError(
                        "climate produced non-finite values".into(),
                    ));
                }

                if updated.planetary_albedo < 0.0 || updated.planetary_albedo > 1.0 {
                    return Err(worldsmith_traits::ContractError::ModuleError(
                        "climate albedo out of bounds".into(),
                    ));
                }

                if updated.greenhouse_temperature_offset_k < 0.0 {
                    return Err(worldsmith_traits::ContractError::ModuleError(
                        "climate greenhouse offset negative".into(),
                    ));
                }
            }

            if let Some(planet) = state.world_mut().planets.get_mut(&planet_id) {
                planet.climate_state = Some(updated);
            }
        }

        Ok(())
    }

    fn shutdown(&mut self, _state: &mut dyn StateWriter) -> ContractResult<()> {
        self.initialized = false;
        Ok(())
    }

    fn reads(&self) -> Vec<FieldKey> {
        vec![
            FieldKey::AtmosphericTemperature,
            FieldKey::AtmosphericPressure,
            FieldKey::AtmosphericComposition,
            FieldKey::TotalWaterMass,
            FieldKey::OceanMass,
            FieldKey::IceMass,
            FieldKey::PlanetMass,
            FieldKey::StellarLuminosity,
        ]
    }

    fn writes(&self) -> Vec<FieldKey> {
        vec![
            FieldKey::EquilibriumTemperature,
            FieldKey::GreenhouseOffset,
            FieldKey::PlanetaryAlbedo,
            FieldKey::ClimateClassification,
        ]
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
    use worldsmith_models::{
        AtmosphereState, ClimateState, HydrologyState, Planet, PlanetId, PlanetType, SystemId,
    };

    #[test]
    fn module_constructs_with_defaults() {
        let _module = ClimateModule::default();
    }

    #[test]
    fn initializes_climate_for_planet() {
        let mut module = ClimateModule::default();
        let mut state =
            worldsmith_state::WorldState::new(worldsmith_state::EngineConfig::default());
        state.planets.insert(PlanetId(1), earth_like_planet());
        state.stars.insert(
            worldsmith_models::StarId(1),
            worldsmith_models::Star {
                id: worldsmith_models::StarId(1),
                name: "Sol".into(),
                spectral_type: worldsmith_models::SpectralType::G,
                class: worldsmith_models::StarClass::MainSequence,
                mass_kg: worldsmith_models::MeasuredValue {
                    value: 1.989e30,
                    unit: "kg".into(),
                    provenance: None,
                },
                radius_m: worldsmith_models::MeasuredValue {
                    value: 6.96e8,
                    unit: "m".into(),
                    provenance: None,
                },
                luminosity_w: worldsmith_models::MeasuredValue {
                    value: 3.828e26,
                    unit: "W".into(),
                    provenance: None,
                },
                effective_temperature_k: worldsmith_models::MeasuredValue {
                    value: 5778.0,
                    unit: "K".into(),
                    provenance: None,
                },
                surface_gravity_m_s2: worldsmith_models::MeasuredValue {
                    value: 274.0,
                    unit: "m/s^2".into(),
                    provenance: None,
                },
                metallicity: worldsmith_models::MeasuredValue {
                    value: 0.0,
                    unit: "dimensionless".into(),
                    provenance: None,
                },
                rotation_period_s: None,
                age_s: None,
                position_m: worldsmith_math::Vector3::ZERO,
                velocity_m_s: worldsmith_math::Vector3::ZERO,
            },
        );
        module.initialize(&mut state).unwrap();

        let planet = state.planets.get(&PlanetId(1)).unwrap();
        let climate = planet.climate_state.as_ref().unwrap();
        assert!(climate.equilibrium_temperature_k > 0.0);
        assert!(climate.planetary_albedo <= 1.0);
    }

    #[test]
    fn frozen_planet_classifies_as_frozen() {
        let mut planet = earth_like_planet();
        planet.atmosphere_state = Some(AtmosphereState {
            atmospheric_mass_kg: 5.15e18,
            surface_pressure_pa: 101_325.0,
            mean_temperature_k: 220.0,
            atmosphere_composition: vec![],
        });
        planet.hydrology_state = Some(HydrologyState {
            total_water_mass_kg: 1.4e21,
            ocean_mass_kg: 0.0,
            atmospheric_water_mass_kg: 0.0,
            ice_mass_kg: 1.4e21,
            liquid_water_fraction: 0.0,
        });

        let mut module = ClimateModule::default();
        let mut state =
            worldsmith_state::WorldState::new(worldsmith_state::EngineConfig::default());
        state.planets.insert(PlanetId(1), planet);
        module.initialize(&mut state).unwrap();

        let planet = state.planets.get(&PlanetId(1)).unwrap();
        let climate = planet.climate_state.as_ref().unwrap();
        assert_eq!(climate.climate_classification, ClimateType::Frozen);
    }

    #[test]
    fn hot_planet_classifies_as_tropical_or_hotter() {
        let mut planet = earth_like_planet();
        planet.atmosphere_state = Some(AtmosphereState {
            atmospheric_mass_kg: 5.15e20,
            surface_pressure_pa: 101_325.0,
            mean_temperature_k: 380.0,
            atmosphere_composition: vec![],
        });
        planet.hydrology_state = Some(HydrologyState {
            total_water_mass_kg: 0.0,
            ocean_mass_kg: 0.0,
            atmospheric_water_mass_kg: 0.0,
            ice_mass_kg: 0.0,
            liquid_water_fraction: 0.0,
        });

        let mut module = ClimateModule::default();
        let mut state =
            worldsmith_state::WorldState::new(worldsmith_state::EngineConfig::default());
        state.planets.insert(PlanetId(1), planet);
        state.stars.insert(
            worldsmith_models::StarId(1),
            worldsmith_models::Star {
                id: worldsmith_models::StarId(1),
                name: "Sol".into(),
                spectral_type: worldsmith_models::SpectralType::G,
                class: worldsmith_models::StarClass::MainSequence,
                mass_kg: worldsmith_models::MeasuredValue {
                    value: 1.989e30,
                    unit: "kg".into(),
                    provenance: None,
                },
                radius_m: worldsmith_models::MeasuredValue {
                    value: 6.96e8,
                    unit: "m".into(),
                    provenance: None,
                },
                luminosity_w: worldsmith_models::MeasuredValue {
                    value: 3.828e26,
                    unit: "W".into(),
                    provenance: None,
                },
                effective_temperature_k: worldsmith_models::MeasuredValue {
                    value: 5778.0,
                    unit: "K".into(),
                    provenance: None,
                },
                surface_gravity_m_s2: worldsmith_models::MeasuredValue {
                    value: 274.0,
                    unit: "m/s^2".into(),
                    provenance: None,
                },
                metallicity: worldsmith_models::MeasuredValue {
                    value: 0.0,
                    unit: "dimensionless".into(),
                    provenance: None,
                },
                rotation_period_s: None,
                age_s: None,
                position_m: worldsmith_math::Vector3::ZERO,
                velocity_m_s: worldsmith_math::Vector3::ZERO,
            },
        );
        module.initialize(&mut state).unwrap();

        let planet = state.planets.get(&PlanetId(1)).unwrap();
        let climate = planet.climate_state.as_ref().unwrap();
        assert!(
            climate.climate_classification == ClimateType::Tropical
                || climate.climate_classification == ClimateType::RunawayGreenhouse
        );
    }

    fn earth_like_planet() -> Planet {
        Planet {
            id: PlanetId(1),
            name: "Earth".into(),
            class: worldsmith_models::PlanetClass::Terrestrial,
            planet_type: PlanetType::Rocky,
            system_id: SystemId(1),
            physical: worldsmith_models::PhysicalProperties {
                mass_kg: worldsmith_models::MeasuredValue {
                    value: 5.972e24,
                    unit: "kg".into(),
                    provenance: None,
                },
                radius_m: worldsmith_models::MeasuredValue {
                    value: 6.371e6,
                    unit: "m".into(),
                    provenance: None,
                },
                density_kg_m3: None,
                surface_gravity_m_s2: None,
            },
            orbit: worldsmith_models::OrbitalProperties {
                parent: worldsmith_models::BodyReference::Star(worldsmith_models::StarId(1)),
                semi_major_axis_m: worldsmith_models::MeasuredValue {
                    value: 1.496e11,
                    unit: "m".into(),
                    provenance: None,
                },
                semi_minor_axis_m: None,
                eccentricity: worldsmith_models::MeasuredValue {
                    value: 0.0167,
                    unit: "dimensionless".into(),
                    provenance: None,
                },
                inclination_rad: worldsmith_models::MeasuredValue {
                    value: 0.0,
                    unit: "rad".into(),
                    provenance: None,
                },
                orbital_period_s: None,
                rotation_period_s: None,
                axial_tilt_rad: None,
            },
            geology: None,
            atmosphere: None,
            atmosphere_state: Some(AtmosphereState::default()),
            hydrology_state: Some(HydrologyState::default()),
            climate_state: Some(ClimateState::default()),
            carbon_cycle_state: None,
            biosphere_state: None,
            habitability_state: None,
            classification_state: None,
            surface_chemistry_state: None,
            cryosphere_state: None,
            interior: None,
            volcanism: None,
            plate_tectonics: None,
            climate: None,
            ocean: None,
            magnetic_field: None,
            habitability: None,
            position_m: worldsmith_math::Vector3::ZERO,
            velocity_m_s: worldsmith_math::Vector3::ZERO,
            moons: vec![],
        }
    }
}
