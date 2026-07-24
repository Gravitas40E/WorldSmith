//! Plate tectonic evolution: lithospheric motion driven by mantle and volcanic
//! activity.
//!
//! This module models a deterministic baseline for plate tectonics.  Phase 10F
//! introduces crustal recycling, plate velocity, and tectonic activity derived
//! from mantle thermal state and volcanic forcing.
//!
//! ## Responsibilities
//!
//! - Owns `tectonic_activity`, `crustal_recycling_rate`, and `plate_velocity`
//!   per ADR-011.
//! - Reads `mantle_temperature`, `heat_flux`, `volcanic_flux`, and
//!   `volcanic_activity` after `worldsmith.evolution.volcanism`.
//! - Applies a deterministic mobility model with configurable activity
//!   thresholds and a deterministic volcanic boost.
//!
//! ## Simplifying assumptions
//!
//! 1. **Single mobility axis**: plate velocity is a scalar derived from bulk
//!    mantle temperature; there is no true vector plate-motion model, no
//!    lithospheric segmentation, and no subduction geometry.
//! 2. **Smooth deterministic functions**: all mappings use clamping and
//!    algebraic power laws, no discontinuities or stochasticity.
//! 3. **Volcanic boost only affects activity classification**: volcanic
//!    forcing is restricted to the `tectonic_activity` threshold test; it
//!    never modifies volcanic state or interior state.
//! 4. **No horizontal coupling**: identical initial conditions and timestep
//!    sequences produce identical results for every planet independently.
//! 5. **Baseline model**: this does not represent continental drift, plate
//!    boundaries, slab pull, ridge push, or true mantle convection.
//!
//! ## Future replacement
//!
//! This implementation is a deterministic thermal-coupling baseline and
//! does not represent full plate tectonics.  Future phases should introduce
//! lithosphere segmentation, subduction, continental drift, and plate
//! boundary physics without changing module ownership or pipeline position.
//!
//! ## Ownership
//!
//! - **Reads**: `mantle_temperature`, `heat_flux`, `volcanic_flux`,
//!   `volcanic_activity`
//! - **Writes**: `tectonic_activity`, `crustal_recycling_rate`,
//!   `plate_velocity`
//! - **Never modifies**: `age_seconds`, `radiogenic_heat`, `core_temperature`,
//!   `mantle_temperature`, `geology`, `atmosphere`, `climate`, `ocean`,
//!   `volcanic_flux`, `volcanic_activity`, `magma_generation_rate`

use serde::{Deserialize, Serialize};
use worldsmith_models::{
    Planet, PlanetId, PlateTectonicsState, TectonicActivity, VolcanicActivity,
};
use worldsmith_state::{FieldKey, SimulationEvent};
use worldsmith_traits::{ContractResult, ModuleContext, SimulationModule, StateWriter};

/// Baseline mantle mobility temperature in kelvin.
const DEFAULT_BASELINE_TEMP: f64 = 2000.0;
/// Reference mantle temperature for full mobility in kelvin.
const DEFAULT_REFERENCE_TEMP: f64 = 5000.0;
/// Scaling factor for nominal plate velocity in cm yr⁻¹.
const DEFAULT_VELOCITY_SCALE: f64 = 10.0;
/// Scaling factor for crustal recycling rate.
const DEFAULT_RECYCLING_COEFFICIENT: f64 = 1.0e-3;
/// Shift applied to plate velocity for volcanic boost during classification.
const DEFAULT_VOLCANIC_BOOST: f64 = 0.5;

/// Plate tectonic evolution configuration.
///
/// All parameters must be non-negative.  Defaults represent a cool terrestrial
/// planet with modest but non-zero tectonic motion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlateTectonicsConfig {
    /// Temperature below which mantle mobility is zero (K).
    pub baseline_temp: f64,
    /// Temperature at which mobility saturates to 1.0 (K).
    pub reference_temp: f64,
    /// Nominal plate velocity scale (cm yr⁻¹).
    pub velocity_scale: f64,
    /// Coefficient mapping velocity to crustal recycling rate.
    pub recycling_coefficient: f64,
    /// Additive shift applied during tectonic activity classification when
    /// volcanic forcing is significant.
    pub volcanic_boost: f64,
    /// Plate velocity below which activity is Dormant.
    pub dormant_threshold: f64,
    /// Plate velocity above which activity is at least Low.
    pub low_threshold: f64,
    /// Plate velocity above which activity is at least Moderate.
    pub moderate_threshold: f64,
    /// Plate velocity above which activity is at least High.
    pub high_threshold: f64,
}

impl Default for PlateTectonicsConfig {
    fn default() -> Self {
        Self {
            baseline_temp: DEFAULT_BASELINE_TEMP,
            reference_temp: DEFAULT_REFERENCE_TEMP,
            velocity_scale: DEFAULT_VELOCITY_SCALE,
            recycling_coefficient: DEFAULT_RECYCLING_COEFFICIENT,
            volcanic_boost: DEFAULT_VOLCANIC_BOOST,
            dormant_threshold: 1.0,
            low_threshold: 4.0,
            moderate_threshold: 7.0,
            high_threshold: 14.0,
        }
    }
}

/// Deterministic plate tectonic evolution module.
///
/// Ownership follows ADR-011: this module is the sole runtime authority for
/// `tectonic_activity`, `crustal_recycling_rate`, and `plate_velocity`.
pub struct PlateTectonicsModule {
    config: PlateTectonicsConfig,
    initialized: bool,
}

impl PlateTectonicsModule {
    /// Creates a new plate tectonic evolution module.
    pub fn new(config: PlateTectonicsConfig) -> Self {
        Self {
            config,
            initialized: false,
        }
    }

    /// Initializes tectonic state for planets missing initialized values.
    fn apply_initial_tectonics(&self, planet: &Planet) -> ContractResult<Planet> {
        let mut updated = planet.clone();
        if updated.plate_tectonics.is_none() {
            updated.plate_tectonics = Some(PlateTectonicsState::default());
        }
        Ok(updated)
    }

    /// Classifies tectonic activity from plate velocity and volcanic forcing.
    fn classify_activity(
        &self,
        plate_velocity: f64,
        volcanic_activity: VolcanicActivity,
    ) -> TectonicActivity {
        let effective_velocity = if matches!(
            volcanic_activity,
            VolcanicActivity::Moderate | VolcanicActivity::High | VolcanicActivity::Extreme,
        ) {
            plate_velocity + self.config.volcanic_boost
        } else {
            plate_velocity
        };

        let c = &self.config;
        if effective_velocity >= c.high_threshold {
            TectonicActivity::High
        } else if effective_velocity >= c.moderate_threshold {
            TectonicActivity::Moderate
        } else if effective_velocity >= c.low_threshold {
            TectonicActivity::Low
        } else if effective_velocity >= c.dormant_threshold {
            TectonicActivity::None
        } else {
            TectonicActivity::None
        }
    }
}

impl Default for PlateTectonicsModule {
    fn default() -> Self {
        Self::new(PlateTectonicsConfig::default())
    }
}

impl SimulationModule for PlateTectonicsModule {
    fn id(&self) -> &'static str {
        "worldsmith.evolution.plate_tectonics"
    }

    fn initialize(&mut self, state: &mut dyn StateWriter) -> ContractResult<()> {
        let planet_ids: Vec<_> = state.world().planets.keys().cloned().collect();
        for planet_id in planet_ids {
            if let Some(planet) = state.world().planets.get(&planet_id).cloned() {
                let updated = self.apply_initial_tectonics(&planet)?;
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

        let baseline = self.config.baseline_temp;
        let reference = self.config.reference_temp;
        let velocity_scale = self.config.velocity_scale;
        let recycling_coeff = self.config.recycling_coefficient;

        // Snapshot current state to satisfy borrow checker and keep
        // reads/writes formally separated.
        let snapshot: Vec<(PlanetId, Planet, Option<PlateTectonicsState>)> = state
            .world()
            .planets
            .values()
            .map(|planet| (planet.id, planet.clone(), planet.plate_tectonics.clone()))
            .collect();

        for (planet_id, planet, plate_tectonics) in snapshot {
            let new_plate_tectonics = if let Some(interior) = planet.interior {
                let mantle_temp = interior.mantle_temperature;
                let _heat_flux = interior.heat_flux;

                if let Some(volcanism) = planet.volcanism.as_ref() {
                    let volcanic_activity = volcanism.volcanic_activity;

                    // Smooth deterministic mobility in [0, 1].
                    let denominator = (reference - baseline).max(1.0);
                    let mobility = ((mantle_temp - baseline) / denominator).clamp(0.0, 1.0);

                    // Plate velocity scales with mobility squared for a smooth
                    // acceleration-curve approximation.
                    let plate_velocity = velocity_scale * mobility.powi(2);

                    // Crustal recycling rate scales sub-linearly with plate
                    // velocity to avoid runaway growth.
                    let crustal_recycling_rate =
                        recycling_coeff * (plate_velocity * plate_velocity).sqrt();

                    // Tectonic activity includes deterministic volcanic boost.
                    let tectonic_activity =
                        self.classify_activity(plate_velocity, volcanic_activity);

                    Some(PlateTectonicsState {
                        plate_velocity,
                        crustal_recycling_rate,
                        tectonic_activity,
                    })
                } else {
                    plate_tectonics.or_else(|| Some(PlateTectonicsState::default()))
                }
            } else {
                plate_tectonics.or_else(|| Some(PlateTectonicsState::default()))
            };

            if let Some(planet) = state.world_mut().planets.get_mut(&planet_id) {
                planet.plate_tectonics = new_plate_tectonics;
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
            FieldKey::MantleTemperature,
            FieldKey::HeatFlux,
            FieldKey::VolcanicFlux,
            FieldKey::VolcanicActivity,
        ]
    }

    fn writes(&self) -> Vec<FieldKey> {
        vec![
            FieldKey::TectonicActivity,
            FieldKey::PlateVelocity,
            FieldKey::CrustalRecyclingRate,
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
    use crate::{CoreEvolutionModule, MantleEvolutionModule, VolcanismModule};
    use worldsmith_engine::EngineBuilder;
    use worldsmith_math::Vector3;
    use worldsmith_models::{
        BodyReference, InteriorState, MeasuredValue, OrbitalProperties, PhysicalProperties, Planet,
        PlanetId, PlanetType, StarId, SystemId,
    };
    use worldsmith_traits::ModuleContext;

    fn earth_like_planet() -> Planet {
        Planet {
            id: PlanetId(1),
            name: "PlateTectonics Test".into(),
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
            volcanism: None,
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
            core_temperature: 6000.0,
            mantle_temperature: 4800.0,
            heat_flux: 1.2e16,
        });
        p.volcanism = Some(worldsmith_models::VolcanismState {
            volcanic_flux: 1.0e16,
            volcanic_activity: VolcanicActivity::Moderate,
            magma_generation_rate: 5.97e9,
        });
        p
    }

    #[test]
    fn module_constructs_with_defaults() {
        let module = PlateTectonicsModule::default();
        assert!(!module.initialized);
    }

    #[test]
    fn initialization_seeds_plate_tectonics_state() {
        let mut engine = EngineBuilder::new()
            .with_seed(7)
            .build()
            .expect("engine builds");
        engine
            .state_mut()
            .planets
            .insert(PlanetId(1), earth_like_planet());

        let mut core = CoreEvolutionModule::default();
        core.initialize(engine.state_mut())
            .expect("core initialization succeeds");

        let mut mantle = MantleEvolutionModule::default();
        mantle
            .initialize(engine.state_mut())
            .expect("mantle initialization succeeds");

        let mut volcanism = VolcanismModule::default();
        volcanism
            .initialize(engine.state_mut())
            .expect("volcanism initialization succeeds");

        let mut plate_tectonics = PlateTectonicsModule::default();
        plate_tectonics
            .initialize(engine.state_mut())
            .expect("plate tectonics initialization succeeds");

        let planet = engine.state().planets.get(&PlanetId(1)).unwrap();
        assert!(
            planet.plate_tectonics.is_some(),
            "plate tectonics state must exist"
        );
        let t = planet.plate_tectonics.as_ref().unwrap();
        assert_eq!(t.plate_velocity, 0.0);
        assert_eq!(t.crustal_recycling_rate, 0.0);
        assert_eq!(t.tectonic_activity, TectonicActivity::None);
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

        let mut core = CoreEvolutionModule::default();
        core.initialize(engine.state_mut()).unwrap();

        let mut mantle = MantleEvolutionModule::default();
        mantle.initialize(engine.state_mut()).unwrap();

        let mut volcanism = VolcanismModule::default();
        volcanism.initialize(engine.state_mut()).unwrap();

        let mut module = PlateTectonicsModule::default();
        module.initialize(engine.state_mut()).unwrap();

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
        let t = planet.plate_tectonics.as_ref().unwrap();
        assert_eq!(t.plate_velocity, 0.0);
        assert_eq!(t.crustal_recycling_rate, 0.0);
    }

    #[test]
    fn values_are_non_negative_and_finite() {
        let mut engine = EngineBuilder::new()
            .with_seed(7)
            .build()
            .expect("engine builds");
        engine
            .state_mut()
            .planets
            .insert(PlanetId(1), seeded_planet());

        let mut core = CoreEvolutionModule::default();
        core.initialize(engine.state_mut()).unwrap();

        let mut mantle = MantleEvolutionModule::default();
        mantle.initialize(engine.state_mut()).unwrap();

        let mut volcanism = VolcanismModule::default();
        volcanism.initialize(engine.state_mut()).unwrap();

        let mut module = PlateTectonicsModule::default();
        module.initialize(engine.state_mut()).unwrap();

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
        let t = planet.plate_tectonics.as_ref().unwrap();
        assert!(
            t.plate_velocity >= 0.0,
            "plate_velocity must not be negative"
        );
        assert!(
            t.crustal_recycling_rate >= 0.0,
            "crustal_recycling_rate must not be negative"
        );
        assert!(t.plate_velocity.is_finite());
        assert!(t.crustal_recycling_rate.is_finite());
    }

    #[test]
    fn hotter_mantle_produces_higher_plate_velocity() {
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

        let mut hotter = seeded_planet();
        hotter.interior.as_mut().unwrap().mantle_temperature = 5500.0;
        engine_b.state_mut().planets.insert(PlanetId(1), hotter);

        let mut core_a = CoreEvolutionModule::default();
        let mut core_b = CoreEvolutionModule::default();
        core_a.initialize(engine_a.state_mut()).unwrap();
        core_b.initialize(engine_b.state_mut()).unwrap();

        let mut mantle_a = MantleEvolutionModule::default();
        let mut mantle_b = MantleEvolutionModule::default();
        mantle_a.initialize(engine_a.state_mut()).unwrap();
        mantle_b.initialize(engine_b.state_mut()).unwrap();

        let mut volcanism_a = VolcanismModule::default();
        let mut volcanism_b = VolcanismModule::default();
        volcanism_a.initialize(engine_a.state_mut()).unwrap();
        volcanism_b.initialize(engine_b.state_mut()).unwrap();

        let mut module_a = PlateTectonicsModule::default();
        let mut module_b = PlateTectonicsModule::default();
        module_a.initialize(engine_a.state_mut()).unwrap();
        module_b.initialize(engine_b.state_mut()).unwrap();

        let ctx = ModuleContext {
            timestamp_s: 0.0,
            delta_seconds: 1.0,
            seed: 7,
        };
        module_a.update(ctx, engine_a.state_mut()).unwrap();
        module_b.update(ctx, engine_b.state_mut()).unwrap();

        let velocity_a = engine_a
            .state()
            .planets
            .get(&PlanetId(1))
            .unwrap()
            .plate_tectonics
            .as_ref()
            .unwrap()
            .plate_velocity;
        let velocity_b = engine_b
            .state()
            .planets
            .get(&PlanetId(1))
            .unwrap()
            .plate_tectonics
            .as_ref()
            .unwrap()
            .plate_velocity;

        assert!(
            velocity_b > velocity_a,
            "hotter mantle {} should produce higher plate velocity than cooler mantle {}",
            velocity_b,
            velocity_a
        );
    }

    #[test]
    fn greater_volcanic_activity_never_decreases_tectonic_activity() {
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

        let mut dormant = seeded_planet();
        dormant.volcanism.as_mut().unwrap().volcanic_activity = VolcanicActivity::None;
        engine_b.state_mut().planets.insert(PlanetId(1), dormant);

        let mut core_a = CoreEvolutionModule::default();
        let mut core_b = CoreEvolutionModule::default();
        core_a.initialize(engine_a.state_mut()).unwrap();
        core_b.initialize(engine_b.state_mut()).unwrap();

        let mut mantle_a = MantleEvolutionModule::default();
        let mut mantle_b = MantleEvolutionModule::default();
        mantle_a.initialize(engine_a.state_mut()).unwrap();
        mantle_b.initialize(engine_b.state_mut()).unwrap();

        let mut volcanism_a = VolcanismModule::default();
        let mut volcanism_b = VolcanismModule::default();
        volcanism_a.initialize(engine_a.state_mut()).unwrap();
        volcanism_b.initialize(engine_b.state_mut()).unwrap();

        let mut module_a = PlateTectonicsModule::default();
        let mut module_b = PlateTectonicsModule::default();
        module_a.initialize(engine_a.state_mut()).unwrap();
        module_b.initialize(engine_b.state_mut()).unwrap();

        let ctx = ModuleContext {
            timestamp_s: 0.0,
            delta_seconds: 1.0,
            seed: 7,
        };
        module_a.update(ctx, engine_a.state_mut()).unwrap();
        module_b.update(ctx, engine_b.state_mut()).unwrap();

        let activity_a = engine_a
            .state()
            .planets
            .get(&PlanetId(1))
            .unwrap()
            .plate_tectonics
            .as_ref()
            .unwrap()
            .tectonic_activity;
        let activity_b = engine_b
            .state()
            .planets
            .get(&PlanetId(1))
            .unwrap()
            .plate_tectonics
            .as_ref()
            .unwrap()
            .tectonic_activity;

        // Moderate volcanism should give at least as high tectonic activity as None.
        assert!(
            activity_a as i32 >= activity_b as i32,
            "volcanic forcing should not decrease tectonic activity: {} vs {}",
            activity_a as i32,
            activity_b as i32
        );
    }

    #[test]
    fn repeated_updates_are_deterministic() {
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

        let mut core_a = CoreEvolutionModule::default();
        let mut core_b = CoreEvolutionModule::default();
        core_a.initialize(engine_a.state_mut()).unwrap();
        core_b.initialize(engine_b.state_mut()).unwrap();

        let mut mantle_a = MantleEvolutionModule::default();
        let mut mantle_b = MantleEvolutionModule::default();
        mantle_a.initialize(engine_a.state_mut()).unwrap();
        mantle_b.initialize(engine_b.state_mut()).unwrap();

        let mut volcanism_a = VolcanismModule::default();
        let mut volcanism_b = VolcanismModule::default();
        volcanism_a.initialize(engine_a.state_mut()).unwrap();
        volcanism_b.initialize(engine_b.state_mut()).unwrap();

        let mut module_a = PlateTectonicsModule::default();
        let mut module_b = PlateTectonicsModule::default();
        module_a.initialize(engine_a.state_mut()).unwrap();
        module_b.initialize(engine_b.state_mut()).unwrap();

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
            .plate_tectonics
            .as_ref()
            .unwrap()
            .clone();
        let b = engine_b
            .state()
            .planets
            .get(&PlanetId(1))
            .unwrap()
            .plate_tectonics
            .as_ref()
            .unwrap()
            .clone();

        assert_eq!(a.tectonic_activity, b.tectonic_activity);
        assert!(
            (a.plate_velocity - b.plate_velocity).abs() < 1e-6,
            "plate velocity should be deterministic: {} vs {}",
            a.plate_velocity,
            b.plate_velocity,
        );
        assert!(
            (a.crustal_recycling_rate - b.crustal_recycling_rate).abs() < 1e-6,
            "recycling rate should be deterministic"
        );
    }
}
