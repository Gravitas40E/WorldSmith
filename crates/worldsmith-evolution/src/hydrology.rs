//! Planetary hydrology: deterministic bulk water reservoir partitioning.
//!
//! This module models a deterministic bulk hydrosphere.  Phase 11B introduces
//! a V1 baseline with no precipitation, rivers, groundwater, or ocean
//! circulation.
//!
//! ## Responsibilities
//!
//! - Owns `total_water_mass_kg`, `ocean_mass_kg`, `atmospheric_water_mass_kg`,
//!   `ice_mass_kg`, and `liquid_water_fraction` per ADR-011.
//! - Reads `mean_temperature_k`, `surface_pressure_pa`,
//!   `atmosphere_composition`, `Planet.physical.mass_kg`, and
//!   `Planet.physical.radius_m` after `worldsmith.evolution.atmosphere`.
//!
//! ## Simplifying assumptions
//!
//! 1. **Bulk reservoirs**: water is partitioned into three planetary-scale
//!    reservoirs (ocean, atmosphere, ice) only.  No latitude, longitude,
//!    altitude, weather, or seasonality.
//! 2. **Temperature-driven partitioning**: reservoir fractions are simple
//!    deterministic functions of the mean temperature provided by the
//!    atmosphere module.
//! 3. **No phase-change kinetics**: transitions between reservoirs obey
//!    static thresholds; no supercooled water, no evaporation delay.
//! 4. **No salinity or composition**: oceans are pure water; no dissolved
//!    gases or minerals.
//! 5. **No external sources or sinks**: water inventory is conserved; no
//!    cometary delivery or photodissociation escape in V1.
//!
//! ## Future replacement
//!
//! This implementation is a deterministic bulk hydrosphere model and does
//! not simulate precipitation, river runoff, groundwater, or ocean
//! circulation.  Future phases should introduce:
//!
//! - precipitation maps
//! - river networks
//! - groundwater aquifers
//! - ocean circulation cells
//!
//! ## Ownership
//!
//! - **Reads**: `mean_temperature_k`, `surface_pressure_pa`,
//!   `atmosphere_composition`, `Planet.physical.mass_kg`,
//!   `Planet.physical.radius_m`
//! - **Writes**: `total_water_mass_kg`, `ocean_mass_kg`,
//!   `atmospheric_water_mass_kg`, `ice_mass_kg`, `liquid_water_fraction`
//! - **Never modifies**: `AtmosphereState`, `InteriorState`,
//!   `VolcanismState`, `PlateTectonicsState`, `climate`, `ocean`,
//!   `magnetic_field`, `habitability`

use serde::{Deserialize, Serialize};
use worldsmith_models::{AtmosphereState, HydrologyState, Planet, PlanetId};
use worldsmith_state::{FieldKey, SimulationEvent};
use worldsmith_traits::{ContractResult, ModuleContext, SimulationModule, StateWriter};

const DEFAULT_FREEZING_POINT: f64 = 273.15;
const DEFAULT_BOILING_POINT: f64 = 373.15;
const DEFAULT_OCEAN_FRACTION_SCALE: f64 = 1.0;
const DEFAULT_EVAPORATION_SCALING: f64 = 0.001;
const DEFAULT_ICE_TRANSITION_WIDTH: f64 = 5.0;

const DEFAULT_TOTAL_WATER_MASS_KG: f64 = 1.4e21;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HydrologyConfig {
    pub freezing_point: f64,
    pub boiling_point: f64,
    pub ocean_fraction_scale: f64,
    pub evaporation_scaling: f64,
    pub ice_transition_width: f64,
}

impl Default for HydrologyConfig {
    fn default() -> Self {
        Self {
            freezing_point: DEFAULT_FREEZING_POINT,
            boiling_point: DEFAULT_BOILING_POINT,
            ocean_fraction_scale: DEFAULT_OCEAN_FRACTION_SCALE,
            evaporation_scaling: DEFAULT_EVAPORATION_SCALING,
            ice_transition_width: DEFAULT_ICE_TRANSITION_WIDTH,
        }
    }
}

pub struct HydrologyModule {
    config: HydrologyConfig,
    initialized: bool,
}

impl HydrologyModule {
    pub fn new(config: HydrologyConfig) -> Self {
        Self {
            config,
            initialized: false,
        }
    }

    fn apply_initial_hydrology(&self, planet: &Planet) -> ContractResult<Planet> {
        let mut updated = planet.clone();
        if updated.hydrology_state.is_none() {
            updated.hydrology_state = Some(HydrologyState {
                total_water_mass_kg: 0.0,
                ocean_mass_kg: 0.0,
                atmospheric_water_mass_kg: 0.0,
                ice_mass_kg: 0.0,
                liquid_water_fraction: 0.0,
            });
        }
        Ok(updated)
    }

    #[allow(clippy::too_many_arguments)]
    fn partition_water(&self, total: f64, temp: f64, _pressure: f64) -> (f64, f64, f64, f64) {
        let freeze = self.config.freezing_point;
        let boil = self.config.boiling_point;
        let width = self.config.ice_transition_width.max(1e-6);

        // Ice fraction: 1 below freeze - width, 0 above freeze + width, linear in between.
        let ice_fraction = if temp <= freeze - width {
            1.0
        } else if temp >= freeze + width {
            0.0
        } else {
            1.0 - (temp - (freeze - width)) / (2.0 * width)
        };

        // Vapor fraction: 0 below or at freeze, scaled linearly up to
        // evaporation_scaling at and above boiling.
        let vapor_fraction = if temp <= freeze {
            0.0
        } else if temp >= boil {
            self.config.evaporation_scaling
        } else {
            self.config.evaporation_scaling * (temp - freeze) / (boil - freeze)
        };

        let ice_fraction = ice_fraction.clamp(0.0, 1.0);
        let vapor_fraction = vapor_fraction.clamp(0.0, 1.0);

        let mut ocean_fraction = (1.0 - ice_fraction - vapor_fraction).max(0.0);
        ocean_fraction *= self.config.ocean_fraction_scale;
        ocean_fraction = ocean_fraction.clamp(0.0, 1.0);

        let ice = total * ice_fraction;
        let atm = total * vapor_fraction;
        let ocean = total * ocean_fraction;

        (ocean, atm, ice, ocean_fraction.max(0.0).min(1.0))
    }
}

impl Default for HydrologyModule {
    fn default() -> Self {
        Self::new(HydrologyConfig::default())
    }
}

impl SimulationModule for HydrologyModule {
    fn id(&self) -> &'static str {
        "worldsmith.evolution.hydrology"
    }

    fn initialize(&mut self, state: &mut dyn StateWriter) -> ContractResult<()> {
        let planet_ids: Vec<_> = state.world().planets.keys().cloned().collect();
        for planet_id in planet_ids {
            if let Some(planet) = state.world().planets.get(&planet_id).cloned() {
                let mut updated = self.apply_initial_hydrology(&planet)?;
                if updated.hydrology_state.is_some() {
                    let temp = planet
                        .atmosphere_state
                        .as_ref()
                        .map(|a| a.mean_temperature_k)
                        .unwrap_or(288.15);

                    let mut state = HydrologyState {
                        total_water_mass_kg: DEFAULT_TOTAL_WATER_MASS_KG,
                        ocean_mass_kg: 0.0,
                        atmospheric_water_mass_kg: 0.0,
                        ice_mass_kg: 0.0,
                        liquid_water_fraction: 0.0,
                    };

                    let (ocean, atm, ice, liquid_frac) =
                        self.partition_water(state.total_water_mass_kg, temp, 0.0);
                    state.ocean_mass_kg = ocean;
                    state.atmospheric_water_mass_kg = atm;
                    state.ice_mass_kg = ice;
                    state.liquid_water_fraction = liquid_frac;

                    updated.hydrology_state = Some(state);
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

        let dt = context.delta_seconds;
        if dt <= 0.0 {
            return Ok(());
        }

        let snapshot: Vec<(
            PlanetId,
            Planet,
            Option<HydrologyState>,
            Option<AtmosphereState>,
        )> = state
            .world()
            .planets
            .values()
            .map(|planet| {
                (
                    planet.id,
                    planet.clone(),
                    planet.hydrology_state.clone(),
                    planet.atmosphere_state.clone(),
                )
            })
            .collect();

        for (planet_id, _planet, hydrology_state, atmosphere_state) in snapshot {
            let Some(prev) = hydrology_state else {
                continue;
            };

            let Some(atmosphere) = atmosphere_state else {
                continue;
            };

            let temp = atmosphere.mean_temperature_k;
            let pressure = atmosphere.surface_pressure_pa;

            let (ocean, atm, ice, liquid_frac) =
                self.partition_water(prev.total_water_mass_kg, temp, pressure);

            let new_ocean = ocean;
            let new_atm = atm;
            let new_ice = ice;
            let new_liquid = liquid_frac;

            // Sanity checks.
            if !prev.total_water_mass_kg.is_finite()
                || !new_ocean.is_finite()
                || !new_atm.is_finite()
                || !new_ice.is_finite()
                || !new_liquid.is_finite()
            {
                return Err(worldsmith_traits::ContractError::ModuleError(
                    "hydrology produced non-finite values".into(),
                ));
            }

            if new_ocean < 0.0 || new_atm < 0.0 || new_ice < 0.0 {
                return Err(worldsmith_traits::ContractError::ModuleError(
                    "hydrology produced negative reservoir mass".into(),
                ));
            }

            if new_liquid < 0.0 || new_liquid > 1.0 {
                return Err(worldsmith_traits::ContractError::ModuleError(
                    "hydrology liquid fraction out of bounds".into(),
                ));
            }

            let conservation_error =
                (new_ocean + new_atm + new_ice - prev.total_water_mass_kg).abs();
            if conservation_error > 1e-3 * prev.total_water_mass_kg.max(1.0) {
                return Err(worldsmith_traits::ContractError::ModuleError(format!(
                    "hydrology reservoir conservation error: {conservation_error}"
                )));
            }

            if let Some(planet) = state.world_mut().planets.get_mut(&planet_id) {
                planet.hydrology_state = Some(HydrologyState {
                    total_water_mass_kg: prev.total_water_mass_kg,
                    ocean_mass_kg: new_ocean,
                    atmospheric_water_mass_kg: new_atm,
                    ice_mass_kg: new_ice,
                    liquid_water_fraction: new_liquid,
                });
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
            FieldKey::PlanetMass,
            FieldKey::SurfaceGravity,
        ]
    }

    fn writes(&self) -> Vec<FieldKey> {
        vec![
            FieldKey::TotalWaterMass,
            FieldKey::OceanMass,
            FieldKey::AtmosphericWaterMass,
            FieldKey::IceMass,
            FieldKey::LiquidWaterFraction,
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
    use worldsmith_models::{AtmosphereState, Planet, PlanetId, PlanetType, SystemId};

    #[test]
    fn initializes_default_total_water_mass() {
        let mut module = HydrologyModule::default();
        let mut state =
            worldsmith_state::WorldState::new(worldsmith_state::EngineConfig::default());
        state.planets.insert(PlanetId(1), earth_like_planet());
        module.initialize(&mut state).unwrap();

        let planet = state.planets.get(&PlanetId(1)).unwrap();
        let hydrology = planet.hydrology_state.as_ref().unwrap();
        assert_eq!(hydrology.total_water_mass_kg, 1.4e21);
        assert!(hydrology.total_water_mass_kg.is_finite());
    }

    fn earth_like_frozen_planet() -> Planet {
        let mut planet = earth_like_planet();
        planet.atmosphere_state = Some(AtmosphereState {
            atmospheric_mass_kg: 5.15e18,
            surface_pressure_pa: 101_325.0,
            mean_temperature_k: 250.0,
            atmosphere_composition: vec![],
        });
        planet.hydrology_state = Some(HydrologyState {
            total_water_mass_kg: 1.4e21,
            ocean_mass_kg: 1.4e21,
            atmospheric_water_mass_kg: 0.0,
            ice_mass_kg: 0.0,
            liquid_water_fraction: 1.0,
        });
        planet
    }

    #[test]
    fn freeze_planet_puts_water_in_ice() {
        let mut module = HydrologyModule::default();
        let mut state =
            worldsmith_state::WorldState::new(worldsmith_state::EngineConfig::default());
        state
            .planets
            .insert(PlanetId(1), earth_like_frozen_planet());
        module.initialize(&mut state).unwrap();
        module
            .update(
                ModuleContext {
                    timestamp_s: 0.0,
                    delta_seconds: 3.154e7,
                    seed: 0,
                },
                &mut state,
            )
            .unwrap();

        let planet = state.planets.get(&PlanetId(1)).unwrap();
        let hydrology = planet.hydrology_state.as_ref().unwrap();
        assert!(hydrology.ice_mass_kg > 0.0);
        assert!(hydrology.liquid_water_fraction < 1.0);
        let reservoir_sum =
            hydrology.ocean_mass_kg + hydrology.ice_mass_kg + hydrology.atmospheric_water_mass_kg;
        assert!(
            (reservoir_sum - hydrology.total_water_mass_kg).abs()
                < 1.0 * hydrology.total_water_mass_kg.max(1.0)
        );
    }

    #[test]
    fn hot_planet_increases_atmospheric_water() {
        let config = HydrologyConfig {
            boiling_point: 300.0,
            ..HydrologyConfig::default()
        };
        let mut module = HydrologyModule::new(config);
        let mut state =
            worldsmith_state::WorldState::new(worldsmith_state::EngineConfig::default());
        state.planets.insert(PlanetId(1), earth_like_planet());
        module.initialize(&mut state).unwrap();
        module
            .update(
                ModuleContext {
                    timestamp_s: 0.0,
                    delta_seconds: 3.154e7,
                    seed: 0,
                },
                &mut state,
            )
            .unwrap();

        let planet = state.planets.get(&PlanetId(1)).unwrap();
        let hydrology = planet.hydrology_state.as_ref().unwrap();
        assert!(hydrology.atmospheric_water_mass_kg > 0.0);
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
                axial_tilt_rad: None,
                rotation_period_s: None,
                orbital_period_s: None,
            },
            geology: None,
            atmosphere: None,
            atmosphere_state: Some(AtmosphereState {
                atmospheric_mass_kg: 5.15e18,
                surface_pressure_pa: 101_325.0,
                mean_temperature_k: 288.15,
                atmosphere_composition: vec![],
            }),
            hydrology_state: Some(HydrologyState {
                total_water_mass_kg: 1.4e21,
                ocean_mass_kg: 1.4e21,
                atmospheric_water_mass_kg: 0.0,
                ice_mass_kg: 0.0,
                liquid_water_fraction: 1.0,
            }),
            climate_state: None,
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
