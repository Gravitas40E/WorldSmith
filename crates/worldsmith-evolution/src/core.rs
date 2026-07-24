//! Core evolution: internal heat budget, cooling, and core state.
//!
//! This module models the long-term thermal evolution of a planet's core.
//! Phase 10C introduces a deterministic baseline implementation — not a
//! complete geophysical model.
//!
//! ## Responsibilities
//!
//! - Owns `InteriorState::core_temperature`, `internal_heat`,
//!   `radiogenic_heat`, and `age_seconds` per ADR-011.
//! - Reads planetary mass to seed initial inventories.
//! - Applies deterministic exponential radioactive decay and Newtonian
//!   cooling on each tick.
//!
//! ## Simplifying assumptions
//!
//! 1. **Exponential decay**: radiogenic heat production follows a fixed
//!    half-life exponential decay. No isotope selection.
//! 2. **Newtonian cooling**: heat loss is proportional to core temperature
//!    with a constant coefficient. No mantle convection detail.
//! 3. **Constant heat capacity**: core temperature is computed as
//!    `seed_temperature + internal_heat / capacity`. No phase change.
//! 4. **No stochasticity**: identical state + timestep sequence produces
//!    bit-for-bit identical results.
//!
//! ## Future replacement
//!
//! This implementation is a deterministic baseline, not a complete
//! geophysical model. Future phases should replace the equations with
//! parameterized radiogenic inventories, temperature-dependent heat
//! capacity, and multi-layer mantle coupling without changing module
//! ownership or pipeline position.

use serde::{Deserialize, Serialize};
use std::f64::consts::LN_2;
use worldsmith_models::{InteriorState, Planet, PlanetId};
use worldsmith_state::{FieldKey, SimulationEvent};
use worldsmith_traits::{ContractResult, ModuleContext, SimulationModule, StateWriter};

/// Half-life of the representative radiogenic isotope in years.
const DECAY_HALF_LIFE_YEARS: f64 = 4.0;
/// Seconds per Julian year approximation.
const SECONDS_PER_YEAR: f64 = 31_557_600.0;
/// Newtonian cooling coefficient (W K⁻¹).
const COOLING_COEFFICIENT: f64 = 1.0e9;
/// Effective heat capacity of the core (J K⁻¹).
const HEAT_CAPACITY: f64 = 1.0e28;
/// Seed radiogenic heat power per Earth-mass (W kg⁻¹).
const RADIOGENIC_HEAT_PER_KG: f64 = 2.0e-15;
/// Seed internal energy per Earth-mass (J kg⁻¹).
const INTERNAL_HEAT_PER_KG: f64 = 1.0e6;

/// Core evolution configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoreEvolutionConfig {
    /// Core temperature in kelvin used as the initial baseline.
    pub initial_core_temperature_k: f64,
}

impl Default for CoreEvolutionConfig {
    fn default() -> Self {
        Self {
            initial_core_temperature_k: 6_000.0,
        }
    }
}

/// Deterministic core evolution module.
///
/// Owns `age_seconds`, `internal_heat`, `radiogenic_heat`, and
/// `core_temperature` according to ADR-011.
pub struct CoreEvolutionModule {
    config: CoreEvolutionConfig,
    initialized: bool,
}

impl CoreEvolutionModule {
    /// Creates a new core evolution module.
    pub fn new(config: CoreEvolutionConfig) -> Self {
        Self {
            config,
            initialized: false,
        }
    }

    /// Initializes interior state for planets missing one.
    fn apply_initial_core(&self, planet: &Planet) -> ContractResult<Planet> {
        let mut updated = planet.clone();
        if updated.interior.is_none() {
            updated.interior = Some(InteriorState::default());
        }
        let interior = updated.interior.as_mut().unwrap();
        interior.core_temperature = self.config.initial_core_temperature_k;
        interior.radiogenic_heat = planet.physical.mass_kg.value * RADIOGENIC_HEAT_PER_KG;
        interior.internal_heat = planet.physical.mass_kg.value * INTERNAL_HEAT_PER_KG;
        interior.age_seconds = 0.0;
        Ok(updated)
    }

    /// Decay constant λ for the exponential radiogenic model.
    ///
    /// λ = ln(2) / t½
    ///
    /// where t½ is the deterministic half-life.
    fn decay_lambda() -> f64 {
        LN_2 / (DECAY_HALF_LIFE_YEARS * SECONDS_PER_YEAR)
    }
}

impl Default for CoreEvolutionModule {
    fn default() -> Self {
        Self::new(CoreEvolutionConfig::default())
    }
}

impl SimulationModule for CoreEvolutionModule {
    fn id(&self) -> &'static str {
        "worldsmith.evolution.core"
    }

    fn initialize(&mut self, state: &mut dyn StateWriter) -> ContractResult<()> {
        let planet_ids: Vec<_> = state.world().planets.keys().cloned().collect();
        for planet_id in planet_ids {
            let planet = state.world().planets.get(&planet_id).cloned();
            if let Some(planet) = planet {
                let updated = self.apply_initial_core(&planet)?;
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

        let lambda = Self::decay_lambda();
        let cooling = COOLING_COEFFICIENT;
        let capacity = HEAT_CAPACITY;
        let seed_temp = self.config.initial_core_temperature_k;

        // Snapshot current interior state to satisfy borrow checker.
        let snapshot: Vec<(PlanetId, Option<InteriorState>)> = state
            .world()
            .planets
            .values()
            .map(|planet| (planet.id, planet.interior.clone()))
            .collect();

        for (planet_id, mut interior) in snapshot {
            if let Some(ref mut interior) = interior {
                // Advance age deterministically.
                interior.age_seconds += dt;

                // Exponential radiogenic decay: H(t) = H₀ e^{-λt}
                interior.radiogenic_heat *= (-lambda * dt).exp();

                // Energy balance: radiogenic input minus Newtonian cooling.
                let heat_loss = cooling * interior.core_temperature;
                interior.internal_heat += (interior.radiogenic_heat - heat_loss) * dt;
                if interior.internal_heat < 0.0 {
                    interior.internal_heat = 0.0;
                }

                // Core temperature from effective heat capacity.
                interior.core_temperature = seed_temp + interior.internal_heat / capacity;
                if !interior.core_temperature.is_finite() {
                    interior.core_temperature = seed_temp;
                }
            }

            if let Some(planet) = state.world_mut().planets.get_mut(&planet_id) {
                planet.interior = interior;
            }
        }

        Ok(())
    }

    fn shutdown(&mut self, _state: &mut dyn StateWriter) -> ContractResult<()> {
        self.initialized = false;
        Ok(())
    }

    fn reads(&self) -> Vec<FieldKey> {
        vec![FieldKey::PlanetMass]
    }

    fn writes(&self) -> Vec<FieldKey> {
        vec![]
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
    use worldsmith_engine::EngineBuilder;
    use worldsmith_math::Vector3;
    use worldsmith_models::{
        BodyReference, MeasuredValue, OrbitalProperties, PhysicalProperties, Planet, PlanetId,
        PlanetType, StarId, SystemId,
    };
    use worldsmith_traits::ModuleContext;

    fn earth_like_planet() -> Planet {
        Planet {
            id: PlanetId(1),
            name: "Core Test".into(),
            class: worldsmith_models::PlanetClass::Terrestrial,
            planet_type: PlanetType::Rocky,
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
                parent: BodyReference::Star(StarId(1)),
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
            interior: None,
            geology: None,
            atmosphere: None,
            climate: None,
            ocean: None,
            magnetic_field: None,
            habitability: None,
            plate_tectonics: None,
            atmosphere_state: None,
            hydrology_state: None,
            climate_state: None,
            carbon_cycle_state: None,
            biosphere_state: None,
            habitability_state: None,
            classification_state: None,
            surface_chemistry_state: None,
            cryosphere_state: None,
            volcanism: None,
            moons: Vec::new(),
            position_m: Vector3::ZERO,
            velocity_m_s: Vector3::ZERO,
        }
    }

    #[test]
    fn module_constructs_with_defaults() {
        let module = CoreEvolutionModule::default();
        assert!(!module.initialized);
    }

    #[test]
    fn initialization_populates_interior_state() {
        let mut engine = EngineBuilder::new()
            .with_seed(7)
            .build()
            .expect("engine builds");
        engine
            .state_mut()
            .planets
            .insert(PlanetId(1), earth_like_planet());
        let mut module = CoreEvolutionModule::default();
        module
            .initialize(engine.state_mut())
            .expect("initialize succeeds");

        let planet = engine.state().planets.get(&PlanetId(1)).unwrap();
        let interior = planet.interior.as_ref().expect("interior present");
        assert_eq!(interior.age_seconds, 0.0);
        assert_eq!(interior.core_temperature, 6_000.0);
        assert_eq!(interior.radiogenic_heat, 5.972e24 * 2.0e-15);
        assert_eq!(interior.internal_heat, 5.972e24 * 1.0e6);
    }

    #[test]
    fn age_advances_correctly() {
        let mut engine = EngineBuilder::new()
            .with_seed(7)
            .build()
            .expect("engine builds");
        engine
            .state_mut()
            .planets
            .insert(PlanetId(1), earth_like_planet());
        let mut module = CoreEvolutionModule::default();
        module
            .initialize(engine.state_mut())
            .expect("initialize succeeds");

        module
            .update(
                ModuleContext {
                    timestamp_s: 0.0,
                    delta_seconds: 1.0,
                    seed: 7,
                },
                engine.state_mut(),
            )
            .expect("update succeeds");

        let planet = engine.state().planets.get(&PlanetId(1)).unwrap();
        let interior = planet.interior.as_ref().unwrap();
        assert_eq!(interior.age_seconds, 1.0);
    }

    #[test]
    fn radiogenic_heat_monotonically_decreases() {
        let mut engine = EngineBuilder::new()
            .with_seed(7)
            .build()
            .expect("engine builds");
        engine
            .state_mut()
            .planets
            .insert(PlanetId(1), earth_like_planet());
        let mut module = CoreEvolutionModule::default();
        module
            .initialize(engine.state_mut())
            .expect("initialize succeeds");

        module
            .update(
                ModuleContext {
                    timestamp_s: 0.0,
                    delta_seconds: 1.0,
                    seed: 7,
                },
                engine.state_mut(),
            )
            .expect("update succeeds");

        let first = engine
            .state()
            .planets
            .get(&PlanetId(1))
            .unwrap()
            .interior
            .as_ref()
            .unwrap()
            .radiogenic_heat;

        module
            .update(
                ModuleContext {
                    timestamp_s: 1.0,
                    delta_seconds: 1.0,
                    seed: 7,
                },
                engine.state_mut(),
            )
            .expect("update succeeds");

        let second = engine
            .state()
            .planets
            .get(&PlanetId(1))
            .unwrap()
            .interior
            .as_ref()
            .unwrap()
            .radiogenic_heat;

        assert!(
            second < first,
            "radiogenic heat must decrease: {second} < {first}"
        );
    }

    #[test]
    fn zero_timestep_produces_no_state_change() {
        let mut engine = EngineBuilder::new()
            .with_seed(7)
            .build()
            .expect("engine builds");
        engine
            .state_mut()
            .planets
            .insert(PlanetId(1), earth_like_planet());
        let mut module = CoreEvolutionModule::default();
        module
            .initialize(engine.state_mut())
            .expect("initialize succeeds");

        module
            .update(
                ModuleContext {
                    timestamp_s: 0.0,
                    delta_seconds: 0.0,
                    seed: 7,
                },
                engine.state_mut(),
            )
            .expect("update succeeds");

        let planet = engine.state().planets.get(&PlanetId(1)).unwrap();
        let interior = planet.interior.as_ref().unwrap();
        assert_eq!(interior.age_seconds, 0.0);
        assert_eq!(interior.radiogenic_heat, 5.972e24 * 2.0e-15);
        assert_eq!(interior.internal_heat, 5.972e24 * 1.0e6);
        assert_eq!(interior.core_temperature, 6_000.0);
    }

    #[test]
    fn repeated_updates_equal_cumulative_elapsed_time() {
        let mut engine_a = EngineBuilder::new()
            .with_seed(7)
            .build()
            .expect("engine builds");
        engine_a
            .state_mut()
            .planets
            .insert(PlanetId(1), earth_like_planet());
        let mut module_a = CoreEvolutionModule::default();
        module_a
            .initialize(engine_a.state_mut())
            .expect("initialize succeeds");

        let mut engine_b = EngineBuilder::new()
            .with_seed(7)
            .build()
            .expect("engine builds");
        engine_b
            .state_mut()
            .planets
            .insert(PlanetId(1), earth_like_planet());
        let mut module_b = CoreEvolutionModule::default();
        module_b
            .initialize(engine_b.state_mut())
            .expect("initialize succeeds");

        module_a
            .update(
                ModuleContext {
                    timestamp_s: 0.0,
                    delta_seconds: 1.0,
                    seed: 7,
                },
                engine_a.state_mut(),
            )
            .expect("update succeeds");
        module_a
            .update(
                ModuleContext {
                    timestamp_s: 1.0,
                    delta_seconds: 2.0,
                    seed: 7,
                },
                engine_a.state_mut(),
            )
            .expect("update succeeds");

        module_b
            .update(
                ModuleContext {
                    timestamp_s: 0.0,
                    delta_seconds: 3.0,
                    seed: 7,
                },
                engine_b.state_mut(),
            )
            .expect("update succeeds");

        let a = engine_a
            .state()
            .planets
            .get(&PlanetId(1))
            .unwrap()
            .interior
            .as_ref()
            .unwrap()
            .clone();
        let b = engine_b
            .state()
            .planets
            .get(&PlanetId(1))
            .unwrap()
            .interior
            .as_ref()
            .unwrap()
            .clone();
        assert_eq!(a, b);
    }

    #[test]
    fn values_remain_finite() {
        let mut engine = EngineBuilder::new()
            .with_seed(7)
            .build()
            .expect("engine builds");
        engine
            .state_mut()
            .planets
            .insert(PlanetId(1), earth_like_planet());
        let mut module = CoreEvolutionModule::default();
        module
            .initialize(engine.state_mut())
            .expect("initialize succeeds");

        for i in 0..100 {
            module
                .update(
                    ModuleContext {
                        timestamp_s: (i as f64) * 1.0,
                        delta_seconds: 1.0,
                        seed: 7,
                    },
                    engine.state_mut(),
                )
                .expect("update succeeds");
        }

        let planet = engine.state().planets.get(&PlanetId(1)).unwrap();
        let interior = planet.interior.as_ref().unwrap();
        assert!(interior.age_seconds.is_finite());
        assert!(interior.radiogenic_heat.is_finite());
        assert!(interior.internal_heat.is_finite());
        assert!(interior.core_temperature.is_finite());
        assert!(interior.core_temperature > 0.0);
    }
}
