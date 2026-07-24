//! Deterministic replay and golden simulation helpers.
//!
//! Given an engine and a planet count, run the simulation and capture a
//! "golden" snapshot.  Re-run with the same configuration and compare
//! bit-for-bit equality.

use std::collections::BTreeMap;

use worldsmith_engine::{Engine, EngineBuilder};
use worldsmith_evolution::{
    CoreEvolutionModule, MantleEvolutionModule, PlateTectonicsModule, VolcanismModule,
};
use worldsmith_math::Vector3;
use worldsmith_models::{
    BodyReference, MeasuredValue, OrbitalProperties, PhysicalProperties, Planet, PlanetId,
    PlanetType, StarId, SystemId,
};
use worldsmith_stellar::StellarModule;

/// Identical outputs from two runs indicate successful deterministic replay.
#[derive(Debug, Clone, PartialEq)]
pub enum ReplayOutcome {
    /// Replay matched the golden output.
    Matches,
    /// Replay diverged.
    Diverged {
        /// Tick at which divergence was detected.
        tick: u64,
    },
}

/// Deterministic "golden" world baseline for regression testing.
///
/// Engineered planet types:
/// - Earth-like (1 Earth mass, 1 AU, rocky)
/// - Mars-like (0.107 Earth mass, 1.52 AU, rocky)
/// - Super Earth (5 Earth masses, 0.8 AU, rocky)
#[derive(Debug, Clone)]
pub struct GoldenWorld {
    /// Planets in this golden world.
    pub planets: Vec<Planet>,
}

impl GoldenWorld {
    /// Builds a static golden world with three representative planets.
    pub fn earth_mars_super_earth() -> Self {
        let earth = Self::earth_like();
        let mars = Self::mars_like();
        let super_earth = Self::super_earth();
        Self {
            planets: vec![earth, mars, super_earth],
        }
    }

    fn earth_like() -> Planet {
        Self::rocky_planet(PlanetId(1), "Earth", 5.972e24, 6.371e6, 1.496e11)
    }
    fn mars_like() -> Planet {
        Self::rocky_planet(PlanetId(2), "Mars", 6.4171e23, 3.3895e6, 1.523679e11)
    }
    fn super_earth() -> Planet {
        Self::rocky_planet(
            PlanetId(3),
            "SuperEarth",
            5.0 * 5.972e24,
            1.5 * 6.371e6,
            0.8 * 1.496e11,
        )
    }

    fn rocky_planet(id: PlanetId, name: &str, mass: f64, radius: f64, semi_major: f64) -> Planet {
        Planet {
            id,
            name: name.into(),
            class: worldsmith_models::PlanetClass::Terrestrial,
            planet_type: PlanetType::Rocky,
            system_id: SystemId(1),
            physical: PhysicalProperties {
                mass_kg: MeasuredValue {
                    value: mass,
                    unit: "kg".into(),
                    provenance: None,
                },
                radius_m: MeasuredValue {
                    value: radius,
                    unit: "m".into(),
                    provenance: None,
                },
                density_kg_m3: None,
                surface_gravity_m_s2: None,
            },
            orbit: OrbitalProperties {
                parent: BodyReference::Star(StarId(1)),
                semi_major_axis_m: MeasuredValue {
                    value: semi_major,
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

    fn clone_planets(&self) -> BTreeMap<PlanetId, Planet> {
        self.planets.iter().map(|p| (p.id, p.clone())).collect()
    }
}

/// Builds an engine seeded with the golden world planets and the Phase 10
/// evolution modules registered.
pub fn build_golden_engine(golden: &GoldenWorld) -> Engine {
    let builder = EngineBuilder::new()
        .with_seed(7)
        .register_module(Box::new(StellarModule::default()))
        .register_module(Box::new(CoreEvolutionModule::default()))
        .register_module(Box::new(MantleEvolutionModule::default()))
        .register_module(Box::new(VolcanismModule::default()))
        .register_module(Box::new(PlateTectonicsModule::default()));

    let mut engine = builder.build().expect("build golden engine");

    for (id, mut planet) in golden.clone_planets() {
        // Ensure no leftover state from previous insertions.
        planet.volcanism = None;
        planet.plate_tectonics = None;
        planet.interior = None;
        engine.state_mut().planets.insert(id, planet);
    }
    engine.initialize().expect("initialize golden engine");
    engine
}

/// Run `target_ticks` on a freshly constructed golden engine and return the
/// final planet states as deterministic blobs (JSON).
pub fn run_golden_simulation(golden: &GoldenWorld, target_ticks: u64) -> serde_json::Value {
    let mut engine = build_golden_engine(golden);
    for _ in 0..target_ticks {
        engine.tick_fixed().expect("tick golden engine");
    }
    snapshots_to_json(&engine.state())
}

/// Compare two JSON snapshots for bit-for-bit equality (string equality on
/// canonical JSON).
pub fn compare_snapshots(a: &serde_json::Value, b: &serde_json::Value) -> ReplayOutcome {
    if a == b {
        ReplayOutcome::Matches
    } else {
        ReplayOutcome::Diverged { tick: 0 }
    }
}

/// Run deterministic replay: construct two identical engines and compare the
/// output after `target_ticks`.
pub fn deterministic_replay(
    golden: &GoldenWorld,
    target_ticks: u64,
) -> (ReplayOutcome, serde_json::Value, serde_json::Value) {
    let a = run_golden_simulation(golden, target_ticks);
    let b = run_golden_simulation(golden, target_ticks);
    let outcome = compare_snapshots(&a, &b);
    (outcome, a, b)
}

fn snapshots_to_json(state: &worldsmith_state::WorldState) -> serde_json::Value {
    serde_json::to_value(state).unwrap_or(serde_json::Value::Null)
}
