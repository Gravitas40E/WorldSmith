//! Runtime orbital dynamics module for planetary and lunar state propagation.
//!
//! This module computes absolute world-space positions and velocities for planets
//! and moons from their orbital elements. It is the single authoritative source
//! of propagated orbital state in the simulation.
//!
//! ## Parent-relative propagation
//!
//! Each body's orbital elements describe motion about a parent body. The module
//! resolves the parent's current position and velocity from `WorldState`, computes
//! the body's Keplerian state via `worldsmith_math::orbital::propagate_orbit_state`,
//! and writes the absolute world-space result back into the model:
//!
//! ```text
//! r_world = r_parent + r_orbit
//! v_world = v_parent + v_orbit
//! ```
//!
//! ## Failure handling
//!
//! Invalid parents, missing orbital elements, or physically impossible parameters
//! are silently skipped. The body retains its current state (typically `Vector3::ZERO`
//! before the first valid update). No panics occur during simulation.
//!
//! ## Events
//!
//! Successful planetary updates publish `EventPayload::OrbitalChanged` with
//! `EventTarget::Planet(id)`. Successful lunar updates publish
//! `EventPayload::OrbitalChanged` with `EventTarget::Moon(id)`.

use serde::{Deserialize, Serialize};
use worldsmith_math::orbital::{kepler_period, propagate_orbit_state, OrbitState};
use worldsmith_math::Vector3;
use worldsmith_models::{BodyReference, MeasuredValue, MoonId, PlanetId};
use worldsmith_state::{
    EventPayload, EventQueue, EventSource, EventTarget, SimulationEvent, WorldState,
};
use worldsmith_traits::{ContractResult, ModuleContext, SimulationModule, StateWriter};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct OrbitalDynamicsModuleConfig {
    /// Maximum iterations for Kepler solver fallback.
    pub kepler_max_iterations: u32,
    /// Convergence tolerance for Kepler solver.
    pub kepler_tolerance: f64,
}

impl Default for OrbitalDynamicsModuleConfig {
    fn default() -> Self {
        Self {
            kepler_max_iterations: 50,
            kepler_tolerance: 1e-12,
        }
    }
}

/// Runtime simulation module that propagates orbital state for planets and moons.
pub struct OrbitalDynamicsModule {
    _config: OrbitalDynamicsModuleConfig,
    initialized: bool,
}

impl OrbitalDynamicsModule {
    /// Creates a new orbital dynamics module.
    pub fn new(config: OrbitalDynamicsModuleConfig) -> Self {
        Self {
            _config: config,
            initialized: false,
        }
    }

    /// Pushes an `OrbitalChanged` event onto the queue.
    fn push_orbital_changed(event_queue: &mut EventQueue, timestamp_s: f64, target: EventTarget) {
        event_queue.push(
            timestamp_s,
            EventSource::Module("worldsmith.orbital".to_string()),
            target.clone(),
            EventPayload::OrbitalChanged { target },
        );
    }

    /// Resolves parent position, velocity, and mass directly from `WorldState`.
    fn resolve_parent(
        world: &WorldState,
        parent: BodyReference,
    ) -> Option<(Vector3, Vector3, f64)> {
        match parent {
            BodyReference::Star(id) => {
                let s = world.stars.get(&id)?;
                Some((s.position_m, s.velocity_m_s, s.mass_kg.value))
            }
            BodyReference::Planet(id) => {
                let p = world.planets.get(&id)?;
                Some((p.position_m, p.velocity_m_s, p.physical.mass_kg.value))
            }
            BodyReference::Moon(id) => {
                let m = world.moons.get(&id)?;
                Some((m.position_m, m.velocity_m_s, m.physical.mass_kg.value))
            }
            BodyReference::Body(_) => None,
        }
    }
}

impl Default for OrbitalDynamicsModule {
    fn default() -> Self {
        Self::new(OrbitalDynamicsModuleConfig::default())
    }
}

impl SimulationModule for OrbitalDynamicsModule {
    fn id(&self) -> &'static str {
        "worldsmith.orbital"
    }

    fn name(&self) -> &'static str {
        "WorldSmith Orbital Dynamics Module"
    }

    fn initialize(&mut self, _state: &mut dyn StateWriter) -> ContractResult<()> {
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

        // Phase 1: read orbit metadata into temporaries. This borrows the maps
        // immutably so we can drop the borrow before mutating bodies.
        let planets: Vec<(PlanetId, BodyReference, f64, f64, f64, Option<f64>)> = {
            let world = state.world();
            world
                .planets
                .iter()
                .filter_map(|(planet_id, planet)| {
                    let sma = planet.orbit.semi_major_axis_m.value;
                    if sma <= 0.0 {
                        return None;
                    }
                    let ecc = planet.orbit.eccentricity.value;
                    if !(0.0..1.0).contains(&ecc) {
                        return None;
                    }
                    let period = if let Some(MeasuredValue { value: p, .. }) =
                        planet.orbit.orbital_period_s
                    {
                        if p <= 0.0 {
                            None
                        } else {
                            Some(p)
                        }
                    } else {
                        None
                    };
                    Some((
                        *planet_id,
                        planet.orbit.parent,
                        sma,
                        ecc,
                        planet.orbit.inclination_rad.value,
                        period,
                    ))
                })
                .collect()
        };

        let moons: Vec<(MoonId, BodyReference, f64, f64, f64, Option<f64>)> = {
            let world = state.world();
            world
                .moons
                .iter()
                .filter_map(|(moon_id, moon)| {
                    let sma = moon.orbit.semi_major_axis_m.value;
                    if sma <= 0.0 {
                        return None;
                    }
                    let ecc = moon.orbit.eccentricity.value;
                    if !(0.0..1.0).contains(&ecc) {
                        return None;
                    }
                    let period =
                        if let Some(MeasuredValue { value: p, .. }) = moon.orbit.orbital_period_s {
                            if p <= 0.0 {
                                None
                            } else {
                                Some(p)
                            }
                        } else {
                            None
                        };
                    Some((
                        *moon_id,
                        moon.orbit.parent,
                        sma,
                        ecc,
                        moon.orbit.inclination_rad.value,
                        period,
                    ))
                })
                .collect()
        };

        let world = state.world_mut();

        // Propagate planetary orbits.
        for (planet_id, parent, sma, ecc, inc, period) in planets {
            let Some((parent_pos, parent_vel, mass)) = Self::resolve_parent(world, parent) else {
                continue;
            };

            let period = match period {
                Some(p) => p,
                None => match kepler_period(mass, sma) {
                    Ok(p) => p,
                    Err(_) => continue,
                },
            };

            if let Ok(OrbitState { position, velocity }) = propagate_orbit_state(mass, sma, ecc, inc, period, context.timestamp_s, None) {
                if let Some(planet) = world.planets.get_mut(&planet_id) {
                    planet.position_m = position + parent_pos;
                    planet.velocity_m_s = velocity + parent_vel;

                    Self::push_orbital_changed(
                        &mut world.event_queue,
                        context.timestamp_s,
                        EventTarget::Planet(planet_id),
                    );
                }
            }
        }

        // Propagate lunar orbits.
        for (moon_id, parent, sma, ecc, inc, period) in moons {
            let Some((parent_pos, parent_vel, mass)) = Self::resolve_parent(&world, parent) else {
                continue;
            };

            let period = match period {
                Some(p) => p,
                None => match kepler_period(mass, sma) {
                    Ok(p) => p,
                    Err(_) => continue,
                },
            };

            if let Ok(OrbitState { position, velocity }) = propagate_orbit_state(mass, sma, ecc, inc, period, context.timestamp_s, None) {
                if let Some(moon) = world.moons.get_mut(&moon_id) {
                    moon.position_m = position + parent_pos;
                    moon.velocity_m_s = velocity + parent_vel;

                    Self::push_orbital_changed(
                        &mut world.event_queue,
                        context.timestamp_s,
                        EventTarget::Moon(moon_id),
                    );
                }
            }
        }

        Ok(())
    }

    fn shutdown(&mut self, _state: &mut dyn StateWriter) -> ContractResult<()> {
        self.initialized = false;
        Ok(())
    }

    fn reads(&self) -> Vec<worldsmith_state::FieldKey> {
        vec![worldsmith_state::FieldKey::OrbitalElements]
    }

    fn writes(&self) -> Vec<worldsmith_state::FieldKey> {
        vec![]
    }

    fn publish_events(&mut self) -> Vec<SimulationEvent> {
        // Events are pushed directly to WorldState.event_queue during update.
        Vec::new()
    }

    fn consume_events(&mut self, _events: &[SimulationEvent]) -> ContractResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worldsmith_engine::EngineConfig;
    use worldsmith_math::orbital::kepler_period;
    use worldsmith_models::*;
    use worldsmith_state::{SimulationClock, WorldState};

    fn solar_mass() -> f64 {
        1.989e30
    }

    fn earth_mass() -> f64 {
        5.972e24
    }

    fn au() -> f64 {
        1.496e11
    }

    fn default_star() -> Star {
        Star {
            id: StarId(1),
            name: "Sun".into(),
            spectral_type: SpectralType::G,
            class: StarClass::MainSequence,
            mass_kg: MeasuredValue {
                value: solar_mass(),
                unit: "kg".into(),
                provenance: None,
            },
            radius_m: MeasuredValue {
                value: 6.957e8,
                unit: "m".into(),
                provenance: None,
            },
            luminosity_w: MeasuredValue {
                value: 3.828e26,
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
                value: 0.0134,
                unit: "dimensionless".into(),
                provenance: None,
            },
            rotation_period_s: None,
            age_s: None,
            position_m: Vector3::ZERO,
            velocity_m_s: Vector3::ZERO,
        }
    }

    fn earth_like_planet(planet_id: PlanetId, star_id: StarId) -> Planet {
        let a = au();
        Planet {
            id: planet_id,
            name: format!("Planet {}", planet_id.0),
            class: PlanetClass::Terrestrial,
            planet_type: PlanetType::Rocky,
            system_id: SystemId(1),
            physical: PhysicalProperties {
                mass_kg: MeasuredValue {
                    value: earth_mass(),
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
                parent: BodyReference::Star(star_id),
                semi_major_axis_m: MeasuredValue {
                    value: a,
                    unit: "m".into(),
                    provenance: None,
                },
                semi_minor_axis_m: None,
                eccentricity: MeasuredValue {
                    value: 0.0,
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
            volcanism: None,
            moons: Vec::new(),
            position_m: Vector3::ZERO,
            velocity_m_s: Vector3::ZERO,
            biosphere_state: None,
            habitability_state: None,
            classification_state: None,
            surface_chemistry_state: None,
            cryosphere_state: None,
        }
    }

    fn make_state_with_star_and_planet() -> WorldState {
        let mut state = WorldState::new(EngineConfig::default());
        state.clock = SimulationClock::new(1.0);
        state.stars.insert(StarId(1), default_star());
        let planet = earth_like_planet(PlanetId(1), StarId(1));
        state.planets.insert(PlanetId(1), planet);
        state
    }

    #[test]
    fn planet_moves_after_update() {
        let mut state = make_state_with_star_and_planet();
        let mut module = OrbitalDynamicsModule::default();
        module.initialize(&mut state).unwrap();

        let before = state.planets.get(&PlanetId(1)).unwrap().position_m;
        module
            .update(
                ModuleContext {
                    timestamp_s: 1.0,
                    delta_seconds: 1.0,
                    seed: 0,
                },
                &mut state,
            )
            .unwrap();
        let after = state.planets.get(&PlanetId(1)).unwrap().position_m;

        assert_ne!(before, after, "planet should move after first update");
    }

    #[test]
    fn circular_orbit_returns_after_one_period() {
        let mut state = make_state_with_star_and_planet();
        let mut module = OrbitalDynamicsModule::default();
        module.initialize(&mut state).unwrap();

        let period = kepler_period(solar_mass(), au()).unwrap();

        // Advance once to establish reference position at t=period.
        module
            .update(
                ModuleContext {
                    timestamp_s: period,
                    delta_seconds: period,
                    seed: 0,
                },
                &mut state,
            )
            .unwrap();

        let start_pos = state.planets.get(&PlanetId(1)).unwrap().position_m;

        // Now advance another full period; should return to the same absolute position.
        module
            .update(
                ModuleContext {
                    timestamp_s: period * 2.0,
                    delta_seconds: period,
                    seed: 0,
                },
                &mut state,
            )
            .unwrap();

        let end_pos = state.planets.get(&PlanetId(1)).unwrap().position_m;
        assert!(
            (end_pos.x - start_pos.x).abs() < 1e-6,
            "circular orbit should return to start after one period, x mismatch: {}",
            end_pos.x - start_pos.x
        );
        assert!(
            (end_pos.y - start_pos.y).abs() < 1e-6,
            "circular orbit should return to start after one period, y mismatch: {}",
            end_pos.y - start_pos.y
        );
        assert!(
            (end_pos.z - start_pos.z).abs() < 1e-6,
            "circular orbit should return to start after one period, z mismatch: {}",
            end_pos.z - start_pos.z
        );
    }

    #[test]
    fn moon_follows_parent_after_update() {
        let mut state = make_state_with_star_and_planet();
        let mut module = OrbitalDynamicsModule::default();
        module.initialize(&mut state).unwrap();

        module
            .update(
                ModuleContext {
                    timestamp_s: 1.0,
                    delta_seconds: 1.0,
                    seed: 0,
                },
                &mut state,
            )
            .unwrap();

        let moon = Moon {
            id: MoonId(1),
            name: "Moon".into(),
            parent: BodyReference::Planet(PlanetId(1)),
            physical: PhysicalProperties {
                mass_kg: MeasuredValue {
                    value: 7.342e22,
                    unit: "kg".into(),
                    provenance: None,
                },
                radius_m: MeasuredValue {
                    value: 1.737e6,
                    unit: "m".into(),
                    provenance: None,
                },
                density_kg_m3: None,
                surface_gravity_m_s2: None,
            },
            orbit: OrbitalProperties {
                parent: BodyReference::Planet(PlanetId(1)),
                semi_major_axis_m: MeasuredValue {
                    value: 3.844e8,
                    unit: "m".into(),
                    provenance: None,
                },
                semi_minor_axis_m: None,
                eccentricity: MeasuredValue {
                    value: 0.0,
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
            atmosphere: None,
            atmosphere_state: None,
            hydrology_state: None,
            climate_state: None,
            carbon_cycle_state: None,
            moons: Vec::new(),
            position_m: Vector3::ZERO,
            velocity_m_s: Vector3::ZERO,
        };
        state.moons.insert(MoonId(1), moon);

        module
            .update(
                ModuleContext {
                    timestamp_s: 2.0,
                    delta_seconds: 1.0,
                    seed: 0,
                },
                &mut state,
            )
            .unwrap();

        let moon_pos = state.moons.get(&MoonId(1)).unwrap().position_m;
        let planet_pos_after = state.planets.get(&PlanetId(1)).unwrap().position_m;
        let offset = moon_pos - planet_pos_after;

        // The offset should be roughly the moon's orbital radius (3.844e8 m).
        let dist = (offset.x.powi(2) + offset.y.powi(2) + offset.z.powi(2)).sqrt();
        assert!(
            (dist - 3.844e8).abs() < 1e6,
            "moon should orbit its parent planet at ~384,400 km, got {} m",
            dist
        );
    }

    #[test]
    fn paused_simulation_produces_no_position_change() {
        let mut state = make_state_with_star_and_planet();
        let mut module = OrbitalDynamicsModule::default();
        module.initialize(&mut state).unwrap();
        module
            .update(
                ModuleContext {
                    timestamp_s: 1.0,
                    delta_seconds: 0.0,
                    seed: 0,
                },
                &mut state,
            )
            .unwrap();
        let pos1 = state.planets.get(&PlanetId(1)).unwrap().position_m;
        module
            .update(
                ModuleContext {
                    timestamp_s: 1.0,
                    delta_seconds: 0.0,
                    seed: 0,
                },
                &mut state,
            )
            .unwrap();
        let pos2 = state.planets.get(&PlanetId(1)).unwrap().position_m;
        assert_eq!(pos1, pos2, "paused simulation should not change position");
    }

    #[test]
    fn invalid_parent_handled_gracefully() {
        let mut state = WorldState::new(EngineConfig::default());
        state.clock = SimulationClock::new(1.0);
        state.stars.insert(StarId(1), default_star());
        // Planet with a non-existent parent.
        let bad_planet = Planet {
            id: PlanetId(1),
            name: "Lost".into(),
            class: PlanetClass::Terrestrial,
            planet_type: PlanetType::Rocky,
            system_id: SystemId(1),
            physical: PhysicalProperties {
                mass_kg: MeasuredValue {
                    value: earth_mass(),
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
                parent: BodyReference::Star(StarId(999)),
                semi_major_axis_m: MeasuredValue {
                    value: au(),
                    unit: "m".into(),
                    provenance: None,
                },
                semi_minor_axis_m: None,
                eccentricity: MeasuredValue {
                    value: 0.0,
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
        };
        state.planets.insert(PlanetId(1), bad_planet);

        let mut module = OrbitalDynamicsModule::default();
        module.initialize(&mut state).unwrap();
        // Should not panic even though the parent is missing.
        module
            .update(
                ModuleContext {
                    timestamp_s: 1.0,
                    delta_seconds: 1.0,
                    seed: 0,
                },
                &mut state,
            )
            .unwrap();

        let pos = state.planets.get(&PlanetId(1)).unwrap().position_m;
        assert_eq!(
            pos,
            Vector3::ZERO,
            "invalid parent should leave position at zero"
        );
    }

    /// Regression: orbital parameter mutations after initialization must be
    /// reflected in the next update, proving no persistent cache hides changes.
    #[test]
    fn parameter_changes_after_initialization_are_reflected() {
        let mut state = make_state_with_star_and_planet();
        let mut module = OrbitalDynamicsModule::default();
        module.initialize(&mut state).unwrap();

        module
            .update(
                ModuleContext {
                    timestamp_s: 1.0,
                    delta_seconds: 1.0,
                    seed: 0,
                },
                &mut state,
            )
            .unwrap();
        let pos_before = state.planets.get(&PlanetId(1)).unwrap().position_m;

        // Mutate semi_major_axis after initialization.
        if let Some(planet) = state.planets.get_mut(&PlanetId(1)) {
            planet.orbit.semi_major_axis_m.value = au() * 2.0;
        }

        module
            .update(
                ModuleContext {
                    timestamp_s: 2.0,
                    delta_seconds: 1.0,
                    seed: 0,
                },
                &mut state,
            )
            .unwrap();
        let pos_after = state.planets.get(&PlanetId(1)).unwrap().position_m;

        assert_ne!(
            pos_before, pos_after,
            "position should change when semi_major_axis changes after initialization"
        );
    }

    /// Regression: parent position/velocity changes between ticks must flow
    /// to children on the next update.
    #[test]
    fn parent_position_change_propagates_to_children() {
        let mut state = make_state_with_star_and_planet();
        let mut module = OrbitalDynamicsModule::default();
        module.initialize(&mut state).unwrap();

        module
            .update(
                ModuleContext {
                    timestamp_s: 1.0,
                    delta_seconds: 1.0,
                    seed: 0,
                },
                &mut state,
            )
            .unwrap();

        let moon = Moon {
            id: MoonId(1),
            name: "Moon".into(),
            parent: BodyReference::Planet(PlanetId(1)),
            physical: PhysicalProperties {
                mass_kg: MeasuredValue {
                    value: 7.342e22,
                    unit: "kg".into(),
                    provenance: None,
                },
                radius_m: MeasuredValue {
                    value: 1.737e6,
                    unit: "m".into(),
                    provenance: None,
                },
                density_kg_m3: None,
                surface_gravity_m_s2: None,
            },
            orbit: OrbitalProperties {
                parent: BodyReference::Planet(PlanetId(1)),
                semi_major_axis_m: MeasuredValue {
                    value: 3.844e8,
                    unit: "m".into(),
                    provenance: None,
                },
                semi_minor_axis_m: None,
                eccentricity: MeasuredValue {
                    value: 0.0,
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
            atmosphere: None,
            atmosphere_state: None,
            hydrology_state: None,
            climate_state: None,
            carbon_cycle_state: None,
            moons: Vec::new(),
            position_m: Vector3::ZERO,
            velocity_m_s: Vector3::ZERO,
        };
        state.moons.insert(MoonId(1), moon);

        // Move the planet to a new position and velocity directly.
        if let Some(planet) = state.planets.get_mut(&PlanetId(1)) {
            planet.position_m = Vector3::new(1e9, 2e9, 3e9);
            planet.velocity_m_s = Vector3::new(100.0, 200.0, 300.0);
        }

        module
            .update(
                ModuleContext {
                    timestamp_s: 2.0,
                    delta_seconds: 1.0,
                    seed: 0,
                },
                &mut state,
            )
            .unwrap();

        let moon_pos = state.moons.get(&MoonId(1)).unwrap().position_m;
        let planet_pos = state.planets.get(&PlanetId(1)).unwrap().position_m;
        let offset = moon_pos - planet_pos;

        let dist = (offset.x.powi(2) + offset.y.powi(2) + offset.z.powi(2)).sqrt();
        assert!(
            (dist - 3.844e8).abs() < 1e6,
            "moon should orbit around updated parent position"
        );
    }

    /// Regression: repeated updates must always consult current WorldState
    /// values, not a prior snapshot.
    #[test]
    fn repeated_updates_use_current_worldstate_values() {
        let mut state = make_state_with_star_and_planet();
        let mut module = OrbitalDynamicsModule::default();
        module.initialize(&mut state).unwrap();

        let mut prev_pos = None;
        for tick in 1..=5 {
            if let Some(planet) = state.planets.get_mut(&PlanetId(1)) {
                planet.orbit.semi_major_axis_m.value = au() * (1.0 + tick as f64 * 0.1);
            }
            module
                .update(
                    ModuleContext {
                        timestamp_s: tick as f64,
                        delta_seconds: 1.0,
                        seed: 0,
                    },
                    &mut state,
                )
                .unwrap();
            let curr = state.planets.get(&PlanetId(1)).unwrap().position_m;
            if let Some(prev) = prev_pos {
                assert_ne!(
                    prev, curr,
                    "position should differ each tick when orbital elements change"
                );
            }
            prev_pos = Some(curr);
        }
    }
}
