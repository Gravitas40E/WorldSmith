//! Volcanic evolution: surface volcanic activity driven by mantle heat.
//!
//! This module models the long-term volcanic activity of a planet.  Phase 10E
//! introduces a deterministic baseline implementation — not a full mantle
//! convection or plate-tectonics model.
//!
//! ## Responsibilities
//!
//! - Owns `volcanic_flux`, `volcanic_activity`, and `magma_generation_rate`
//!   per ADR-011.
//! - Reads `mantle_temperature` and `heat_flux` after
//!   `worldsmith.evolution.mantle`.
//! - Applies a deterministic melt-fraction model with configurable
//!   activity thresholds.
//!
//! ## Simplifying assumptions
//!
//! 1. **Melt fraction**: only the temperature excess above a configurable
//!    `melt_temperature` contributes to magma generation.
//! 2. **Single bulk mantle temperature**: no depth stratification or
//!    convective cell detail.
//! 3. **Deterministic thresholds**: classification into Dormant / Low /
//!    Moderate / High / Extreme uses fixed configurable cutoffs.
//! 4. **No stochasticity**: identical state + timestep sequence produces
//!    bit-for-bit identical results.
//! 5. **No tectonic interaction**: volcanism is driven purely by thermal
//!    state, not plate boundaries or lithospheric stress.
//!
//! ## Future replacement
//!
//! This implementation is a deterministic thermal-coupling baseline and
//! does not represent full mantle convection, partial melting curves, or
//! tectonic controls.  Future phases should introduce depth-resolved
//! melting, volatile budgets, and plate-tectonic forcing without changing
//! module ownership or pipeline position.
//!
//! ## Ownership
//!
//! - **Reads**: `mantle_temperature`, `heat_flux`
//! - **Writes**: `volcanic_flux`, `volcanic_activity`, `magma_generation_rate`
//! - **Never modifies**: `age_seconds`, `radiogenic_heat`, `core_temperature`,
//!   `mantle_temperature`, `geology`, `atmosphere`, `climate`, `ocean`

use serde::{Deserialize, Serialize};
use worldsmith_models::{Planet, PlanetId, VolcanicActivity, VolcanismState};
use worldsmith_state::{FieldKey, SimulationEvent};
use worldsmith_traits::{ContractResult, ModuleContext, SimulationModule, StateWriter};

/// Melting threshold in kelvin.
const DEFAULT_MELT_TEMPERATURE: f64 = 4000.0;
/// Scaling factor for magma generation rate (kg s⁻¹ per kg per K).
const DEFAULT_ERUPTION_SCALING: f64 = 1.0e-15;
/// Coefficient linking mantle heat flux to volcanic flux.
const DEFAULT_FLUX_COEFFICIENT: f64 = 1.0;

/// Volcanic evolution configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VolcanismConfig {
    /// Temperature above which mantle material begins to melt (K).
    pub melt_temperature: f64,
    /// Scaling constant for magma generation rate.
    pub eruption_scaling: f64,
    /// Coefficient for volcanic flux computation.
    pub flux_coefficient: f64,
    /// Volcanic flux below which activity is Dormant.
    pub dormant_threshold: f64,
    /// Volcanic flux above which activity is at least Low.
    pub low_threshold: f64,
    /// Volcanic flux above which activity is at least Moderate.
    pub moderate_threshold: f64,
    /// Volcanic flux above which activity is at least High.
    pub high_threshold: f64,
}

impl Default for VolcanismConfig {
    fn default() -> Self {
        Self {
            melt_temperature: DEFAULT_MELT_TEMPERATURE,
            eruption_scaling: DEFAULT_ERUPTION_SCALING,
            flux_coefficient: DEFAULT_FLUX_COEFFICIENT,
            dormant_threshold: 1.0e12,
            low_threshold: 1.0e14,
            moderate_threshold: 1.0e16,
            high_threshold: 1.0e18,
        }
    }
}

/// Deterministic volcanic evolution module.
///
/// Owns `volcanic_flux`, `volcanic_activity`, and `magma_generation_rate`
/// according to ADR-011.
pub struct VolcanismModule {
    config: VolcanismConfig,
    initialized: bool,
}

impl VolcanismModule {
    /// Creates a new volcanic evolution module.
    pub fn new(config: VolcanismConfig) -> Self {
        Self {
            config,
            initialized: false,
        }
    }

    /// Initializes volcanic state for planets missing initialized values.
    fn apply_initial_volcanism(&self, planet: &Planet) -> ContractResult<Planet> {
        let mut updated = planet.clone();
        if updated.volcanism.is_none() {
            updated.volcanism = Some(VolcanismState::default());
        }
        Ok(updated)
    }

    /// Classifies volcanic activity from a deterministic flux magnitude.
    fn classify_activity(&self, flux: f64) -> VolcanicActivity {
        let c = &self.config;
        if flux >= c.high_threshold {
            VolcanicActivity::Extreme
        } else if flux >= c.moderate_threshold {
            VolcanicActivity::High
        } else if flux >= c.low_threshold {
            VolcanicActivity::Moderate
        } else if flux >= c.dormant_threshold {
            VolcanicActivity::Low
        } else {
            VolcanicActivity::None
        }
    }
}

impl Default for VolcanismModule {
    fn default() -> Self {
        Self::new(VolcanismConfig::default())
    }
}

impl SimulationModule for VolcanismModule {
    fn id(&self) -> &'static str {
        "worldsmith.evolution.volcanism"
    }

    fn initialize(&mut self, state: &mut dyn StateWriter) -> ContractResult<()> {
        let planet_ids: Vec<_> = state.world().planets.keys().cloned().collect();
        for planet_id in planet_ids {
            if let Some(planet) = state.world().planets.get(&planet_id).cloned() {
                let updated = self.apply_initial_volcanism(&planet)?;
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

        let melt_temp = self.config.melt_temperature;
        let eruption_scaling = self.config.eruption_scaling;
        let flux_coeff = self.config.flux_coefficient;

        // Snapshot current state to satisfy borrow checker and keep
        // reads/writes formally separated.
        let snapshot: Vec<(PlanetId, Planet, Option<VolcanismState>)> = state
            .world()
            .planets
            .values()
            .map(|planet| (planet.id, planet.clone(), planet.volcanism.clone()))
            .collect();

        for (planet_id, planet, volcanism) in snapshot {
            let new_volcanism = if let Some(interior) = planet.interior {
                let t_mantle = interior.mantle_temperature;
                let heat_flux = interior.heat_flux;
                let core_temp = interior.core_temperature;
                let mass = planet.physical.mass_kg.value;

                // Deterministic melt fraction based on excess above melt temperature.
                let excess = (t_mantle - melt_temp).max(0.0);
                let max_excess = (core_temp - melt_temp).max(1.0);
                let melt_fraction = (excess / max_excess).clamp(0.0, 1.0);

                // Magma generation rate scales with mass and melt fraction.
                let magma_generation_rate = eruption_scaling * mass * melt_fraction;

                // Volcanic flux is a deterministic function of heat flux and
                // melt fraction.
                let volcanic_flux = flux_coeff * heat_flux * melt_fraction;

                // Classify activity from flux magnitude.
                let volcanic_activity = self.classify_activity(volcanic_flux);

                Some(VolcanismState {
                    volcanic_flux,
                    volcanic_activity,
                    magma_generation_rate,
                })
            } else {
                volcanism.or_else(|| Some(VolcanismState::default()))
            };

            if let Some(planet) = state.world_mut().planets.get_mut(&planet_id) {
                planet.volcanism = new_volcanism;
            }
        }

        Ok(())
    }

    fn shutdown(&mut self, _state: &mut dyn StateWriter) -> ContractResult<()> {
        self.initialized = false;
        Ok(())
    }

    fn reads(&self) -> Vec<FieldKey> {
        vec![FieldKey::MantleTemperature, FieldKey::HeatFlux]
    }

    fn writes(&self) -> Vec<FieldKey> {
        vec![
            FieldKey::VolcanicFlux,
            FieldKey::VolcanicActivity,
            FieldKey::MagmaGenerationRate,
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
    use crate::{CoreEvolutionModule, MantleEvolutionModule};
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
            name: "Volcanism Test".into(),
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
            core_temperature: 6000.0,
            mantle_temperature: 4800.0,
            heat_flux: 1.2e16,
        });
        p
    }

    #[test]
    fn module_constructs_with_defaults() {
        let module = VolcanismModule::default();
        assert!(!module.initialized);
    }

    #[test]
    fn initialization_seeds_volcanism_state() {
        let mut engine = EngineBuilder::new()
            .with_seed(7)
            .build()
            .expect("engine builds");
        engine
            .state_mut()
            .planets
            .insert(PlanetId(1), earth_like_planet());

        // Run core then mantle before volcanism.
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

        let planet = engine.state().planets.get(&PlanetId(1)).unwrap();
        assert!(planet.volcanism.is_some(), "volcanism state must exist");
        let v = planet.volcanism.as_ref().unwrap();
        assert_eq!(v.volcanic_flux, 0.0);
        assert_eq!(v.magma_generation_rate, 0.0);
        assert_eq!(v.volcanic_activity, VolcanicActivity::None);
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

        let mut module = VolcanismModule::default();
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
        let v = planet.volcanism.as_ref().unwrap();
        assert_eq!(v.volcanic_flux, 0.0);
        assert_eq!(v.magma_generation_rate, 0.0);
    }

    #[test]
    fn volcanic_flux_is_never_negative() {
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

        let mut module = VolcanismModule::default();
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
        let v = planet.volcanism.as_ref().unwrap();
        assert!(v.volcanic_flux >= 0.0, "volcanic_flux must not be negative");
        assert!(
            v.magma_generation_rate >= 0.0,
            "magma_generation_rate must not be negative"
        );
        assert!(v.volcanic_flux.is_finite());
        assert!(v.magma_generation_rate.is_finite());
    }

    #[test]
    fn hotter_mantle_produces_higher_volcanic_flux() {
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

        let mut module_a = VolcanismModule::default();
        let mut module_b = VolcanismModule::default();
        module_a.initialize(engine_a.state_mut()).unwrap();
        module_b.initialize(engine_b.state_mut()).unwrap();

        let ctx = ModuleContext {
            timestamp_s: 0.0,
            delta_seconds: 1.0,
            seed: 7,
        };
        module_a.update(ctx, engine_a.state_mut()).unwrap();
        module_b.update(ctx, engine_b.state_mut()).unwrap();

        let flux_a = engine_a
            .state()
            .planets
            .get(&PlanetId(1))
            .unwrap()
            .volcanism
            .as_ref()
            .unwrap()
            .volcanic_flux;
        let flux_b = engine_b
            .state()
            .planets
            .get(&PlanetId(1))
            .unwrap()
            .volcanism
            .as_ref()
            .unwrap()
            .volcanic_flux;

        assert!(
            flux_b > flux_a,
            "hotter mantle {} should produce more flux than cooler mantle {}",
            flux_b,
            flux_a
        );
    }

    #[test]
    fn zero_heat_flux_produces_dormant_volcanism() {
        let mut engine = EngineBuilder::new()
            .with_seed(7)
            .build()
            .expect("engine builds");
        let mut planet = seeded_planet();
        planet.interior.as_mut().unwrap().heat_flux = 0.0;
        engine.state_mut().planets.insert(PlanetId(1), planet);

        let mut core = CoreEvolutionModule::default();
        core.initialize(engine.state_mut()).unwrap();

        let mut mantle = MantleEvolutionModule::default();
        mantle.initialize(engine.state_mut()).unwrap();

        let mut module = VolcanismModule::default();
        module.initialize(engine.state_mut()).unwrap();

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
        let v = planet.volcanism.as_ref().unwrap();
        assert_eq!(v.volcanic_activity, VolcanicActivity::None);
        assert_eq!(v.volcanic_flux, 0.0);
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

        let mut core_a = CoreEvolutionModule::default();
        let mut core_b = CoreEvolutionModule::default();
        core_a.initialize(engine_a.state_mut()).unwrap();
        core_b.initialize(engine_b.state_mut()).unwrap();

        let mut mantle_a = MantleEvolutionModule::default();
        let mut mantle_b = MantleEvolutionModule::default();
        mantle_a.initialize(engine_a.state_mut()).unwrap();
        mantle_b.initialize(engine_b.state_mut()).unwrap();

        let mut module_a = VolcanismModule::default();
        let mut module_b = VolcanismModule::default();
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
            .volcanism
            .as_ref()
            .unwrap()
            .clone();
        let b = engine_b
            .state()
            .planets
            .get(&PlanetId(1))
            .unwrap()
            .volcanism
            .as_ref()
            .unwrap()
            .clone();

        assert_eq!(a.volcanic_activity, b.volcanic_activity);
        assert!((a.magma_generation_rate - b.magma_generation_rate).abs() < 1e-6);
        let rel_diff = (a.volcanic_flux - b.volcanic_flux).abs()
            / (a.volcanic_flux + b.volcanic_flux).abs().max(1.0);
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

        let mut core = CoreEvolutionModule::default();
        core.initialize(engine.state_mut()).unwrap();

        let mut mantle = MantleEvolutionModule::default();
        mantle.initialize(engine.state_mut()).unwrap();

        let mut module = VolcanismModule::default();
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
        let v = planet.volcanism.as_ref().unwrap();
        assert!(v.volcanic_flux.is_finite());
        assert!(v.magma_generation_rate.is_finite());
    }
}
