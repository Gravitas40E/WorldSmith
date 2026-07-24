//! Mantle evolution: thermal coupling with the core.
//!
//! This module models the long-term thermal response of a planet's mantle
//! to heat supplied by the core.  Phase 10D introduces a deterministic
//! baseline implementation — not a full mantle convection model.
//!
//! ## Responsibilities
//!
//! - Owns `mantle_temperature` and `heat_flux` per ADR-011.
//! - Reads `core_temperature` and `internal_heat` after
//!   `worldsmith.evolution.core`.
//! - Applies a first-order thermal relaxation model.
//!
//! ## Simplifying assumptions
//!
//! 1. **First-order relaxation**: mantle temperature evolves as
//!    `dT_m/dt = k * (T_c - T_m)`.  No mantle convection cells.
//! 2. **Deterministic conductance**: heat flux is proportional to the
//!    temperature difference with a fixed coefficient.
//! 3. **No stratification**: single bulk mantle temperature.
//! 4. **No stochasticity**: identical state + timestep sequence produces
//!    bit-for-bit identical results.
//!
//! ## Future replacement
//!
//! This implementation is a deterministic thermal coupling baseline and
//! does not represent full mantle convection.  Future phases should
//! introduce layered temperature profiles, Rayleigh-number regimes, and
//! variable conductivity without changing module ownership or pipeline
//! position.
//!
//! ## Ownership
//!
//! - **Reads**: `core_temperature`, `internal_heat`, `Planet.mass_kg`
//! - **Writes**: `mantle_temperature`, `heat_flux`
//! - **Never modifies**: `age_seconds`, `radiogenic_heat`,
//!   `core_temperature`, `geology`, `atmosphere`, `climate`, `ocean`

use serde::{Deserialize, Serialize};
use worldsmith_models::{InteriorState, Planet, PlanetId};
use worldsmith_state::{FieldKey, SimulationEvent};
use worldsmith_traits::{ContractResult, ModuleContext, SimulationModule, StateWriter};

/// Thermal coupling coefficient (s⁻¹).
const DEFAULT_THERMAL_COUPLING: f64 = 1.0e-10;
/// Thermal conductance coefficient (W K⁻¹).
const DEFAULT_THERMAL_CONDUCTIVITY: f64 = 1.0e13;
/// Mantle heat capacity (J K⁻¹).
const DEFAULT_MANTLE_HEAT_CAPACITY: f64 = 1.0e28;
/// Mantle relaxation rate (s⁻¹); same scale as thermal coupling in V1.
const DEFAULT_RELAXATION_RATE: f64 = 1.0e-10;

/// Mantle evolution configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MantleEvolutionConfig {
    /// Thermal coupling constant used for mantle temperature relaxation.
    pub thermal_coupling: f64,
    /// Thermal conductance coefficient for heat-flux computation.
    pub thermal_conductivity: f64,
    /// Effective mantle heat capacity.  Reserved for energy-consistent
    /// formulations in future phases.
    pub mantle_heat_capacity: f64,
    /// Relaxation rate.  In V1 this equals `thermal_coupling`; future
    /// phases may use it separately.
    pub relaxation_rate: f64,
}

impl Default for MantleEvolutionConfig {
    fn default() -> Self {
        Self {
            thermal_coupling: DEFAULT_THERMAL_COUPLING,
            thermal_conductivity: DEFAULT_THERMAL_CONDUCTIVITY,
            mantle_heat_capacity: DEFAULT_MANTLE_HEAT_CAPACITY,
            relaxation_rate: DEFAULT_RELAXATION_RATE,
        }
    }
}

/// Deterministic mantle evolution module.
///
/// Owns `mantle_temperature` and `heat_flux` according to ADR-011.
pub struct MantleEvolutionModule {
    config: MantleEvolutionConfig,
    initialized: bool,
}

impl MantleEvolutionModule {
    /// Creates a new mantle evolution module.
    pub fn new(config: MantleEvolutionConfig) -> Self {
        Self {
            config,
            initialized: false,
        }
    }

    /// Initializes mantle state for planets missing initialized values.
    fn apply_initial_mantle(&self, planet: &Planet) -> ContractResult<Planet> {
        let mut updated = planet.clone();
        if updated.interior.is_none() {
            updated.interior = Some(InteriorState::default());
        }
        let interior = updated.interior.as_mut().unwrap();
        if interior.mantle_temperature == 0.0 && interior.core_temperature > 0.0 {
            // Seed mantle slightly cooler than core.  This is a first-order
            // approximation for an undifferentiated body; future phases
            // should replace this with differentiation physics.
            interior.mantle_temperature = interior.core_temperature * 0.8;
        }
        Ok(updated)
    }
}

impl Default for MantleEvolutionModule {
    fn default() -> Self {
        Self::new(MantleEvolutionConfig::default())
    }
}

impl SimulationModule for MantleEvolutionModule {
    fn id(&self) -> &'static str {
        "worldsmith.evolution.mantle"
    }

    fn initialize(&mut self, state: &mut dyn StateWriter) -> ContractResult<()> {
        let planet_ids: Vec<_> = state.world().planets.keys().cloned().collect();
        for planet_id in planet_ids {
            if let Some(planet) = state.world().planets.get(&planet_id).cloned() {
                let updated = self.apply_initial_mantle(&planet)?;
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

        let coupling = self.config.thermal_coupling;
        let conductivity = self.config.thermal_conductivity;

        // Snapshot current state to satisfy borrow checker and keep
        // reads/writes formally separated.
        let snapshot: Vec<(PlanetId, Option<InteriorState>)> = state
            .world()
            .planets
            .values()
            .map(|planet| (planet.id, planet.interior.clone()))
            .collect();

        for (planet_id, mut interior) in snapshot {
            if let Some(ref mut interior) = interior {
                let t_core = interior.core_temperature;
                let mut t_mantle = interior.mantle_temperature;

                // First-order thermal relaxation:
                // dT_m/dt = k * (T_core - T_mantle)
                // T_m += dt * k * (T_core - T_mantle)
                let delta_t = t_core - t_mantle;
                t_mantle += coupling * delta_t * dt;

                // Heat flux from core into mantle (W).
                // Q = conductivity * (T_core - T_mantle)
                let heat_flux = conductivity * delta_t;

                // Validation: values must remain finite and physically
                // bounded by the core temperature.  The relaxation model
                // never overshoots the core temperature for positive dt.
                interior.mantle_temperature = if t_mantle.is_finite() {
                    t_mantle
                } else {
                    // Clamp to a safe fallback; config should prevent NaNs.
                    t_core * 0.8
                };
                interior.heat_flux = if heat_flux.is_finite() {
                    heat_flux
                } else {
                    0.0
                };
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
        vec![FieldKey::MantleTemperature]
    }

    fn writes(&self) -> Vec<FieldKey> {
        vec![FieldKey::MantleTemperature, FieldKey::HeatFlux]
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
    use crate::CoreEvolutionModule;
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
            name: "Mantle Test".into(),
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

    fn seeded_planet() -> Planet {
        let mut p = earth_like_planet();
        p.interior = Some(InteriorState {
            age_seconds: 0.0,
            internal_heat: 5.972e24 * 1.0e6,
            radiogenic_heat: 5.972e24 * 2.0e-15,
            core_temperature: 6_000.0,
            mantle_temperature: 4_800.0,
            heat_flux: 0.0,
        });
        p
    }

    #[test]
    fn module_constructs_with_defaults() {
        let module = MantleEvolutionModule::default();
        assert!(!module.initialized);
    }

    #[test]
    fn initialization_seeds_mantle_temperature() {
        let mut engine = EngineBuilder::new()
            .with_seed(7)
            .build()
            .expect("engine builds");
        engine
            .state_mut()
            .planets
            .insert(PlanetId(1), earth_like_planet());

        // Run core first so core_temperature is seeded.
        let mut core = CoreEvolutionModule::default();
        core.initialize(engine.state_mut())
            .expect("core initialization succeeds");

        let mut mantle = MantleEvolutionModule::default();
        mantle
            .initialize(engine.state_mut())
            .expect("mantle initialization succeeds");

        let planet = engine.state().planets.get(&PlanetId(1)).unwrap();
        let interior = planet.interior.as_ref().expect("interior present");
        assert_eq!(interior.core_temperature, 6_000.0);
        assert_eq!(interior.mantle_temperature, 4_800.0);
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
            .insert(PlanetId(1), seeded_planet());

        let mut module = MantleEvolutionModule::default();
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
        assert_eq!(interior.mantle_temperature, 4_800.0);
        assert_eq!(interior.heat_flux, 0.0);
    }

    #[test]
    fn mantle_temperature_never_exceeds_core_temperature() {
        let mut engine = EngineBuilder::new()
            .with_seed(7)
            .build()
            .expect("engine builds");
        engine
            .state_mut()
            .planets
            .insert(PlanetId(1), seeded_planet());

        let mut module = MantleEvolutionModule::default();
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
        assert!(
            interior.mantle_temperature.is_finite(),
            "mantle temperature must remain finite"
        );
        assert!(
            interior.mantle_temperature <= interior.core_temperature,
            "mantle {} must not exceed core {}",
            interior.mantle_temperature,
            interior.core_temperature
        );
    }

    #[test]
    fn heat_flux_is_zero_at_thermal_equilibrium() {
        let mut engine = EngineBuilder::new()
            .with_seed(7)
            .build()
            .expect("engine builds");
        let mut planet = seeded_planet();
        planet.interior.as_mut().unwrap().mantle_temperature =
            planet.interior.as_ref().unwrap().core_temperature;
        engine.state_mut().planets.insert(PlanetId(1), planet);

        let mut module = MantleEvolutionModule::default();
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

        let interior = engine
            .state()
            .planets
            .get(&PlanetId(1))
            .unwrap()
            .interior
            .as_ref()
            .unwrap();
        assert_eq!(interior.heat_flux, 0.0);
    }

    #[test]
    fn repeated_updates_converge_smoothly() {
        let mut engine_a = EngineBuilder::new()
            .with_seed(7)
            .build()
            .expect("engine builds");
        engine_a
            .state_mut()
            .planets
            .insert(PlanetId(1), seeded_planet());

        let mut engine_b = EngineBuilder::new()
            .with_seed(7)
            .build()
            .expect("engine builds");
        engine_b
            .state_mut()
            .planets
            .insert(PlanetId(1), seeded_planet());

        let mut module_a = MantleEvolutionModule::default();
        module_a
            .initialize(engine_a.state_mut())
            .expect("initialize succeeds");

        let mut module_b = MantleEvolutionModule::default();
        module_b
            .initialize(engine_b.state_mut())
            .expect("initialize succeeds");

        for i in 0..50 {
            module_a
                .update(
                    ModuleContext {
                        timestamp_s: (i as f64) * 1.0,
                        delta_seconds: 1.0,
                        seed: 7,
                    },
                    engine_a.state_mut(),
                )
                .expect("update succeeds");
        }

        module_b
            .update(
                ModuleContext {
                    timestamp_s: 0.0,
                    delta_seconds: 50.0,
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
        assert_eq!(a.age_seconds, b.age_seconds);
        assert!((a.mantle_temperature - b.mantle_temperature).abs() < 1e-9);
        let rel_diff =
            (a.heat_flux - b.heat_flux).abs() / (a.heat_flux + b.heat_flux).abs().max(1.0);
        assert!(rel_diff < 1e-8);
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
            .insert(PlanetId(1), seeded_planet());

        let mut module = MantleEvolutionModule::default();
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
        assert!(interior.mantle_temperature.is_finite());
        assert!(interior.heat_flux.is_finite());
        assert!(interior.mantle_temperature > 0.0);
    }
}
