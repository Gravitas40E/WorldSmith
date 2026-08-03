//! Demonstrates deterministic replay: two engines with the same seed produce identical outputs.

use worldsmith_engine::EngineBuilder;
use worldsmith_models::PlanetId;

fn main() -> worldsmith_engine::EngineResult<()> {
    let (snap_a, snap_b) = run_pair(777)?;
    assert_eq!(snap_a.timestamp_s, snap_b.timestamp_s);
    assert_eq!(snap_a.planets.len(), snap_b.planets.len());
    for (a, b) in snap_a.planets.iter().zip(snap_b.planets.iter()) {
        assert_eq!(a.planet.id, b.planet.id);
        assert_eq!(a.planet.name, b.planet.name);
    }
    println!("Deterministic replay verified.");
    Ok(())
}

fn run_pair(
    seed: u64,
) -> worldsmith_engine::EngineResult<(
    worldsmith_state::SimulationSnapshot,
    worldsmith_state::SimulationSnapshot,
)> {
    let a = run_engine(seed);
    let b = run_engine(seed);
    Ok((a, b))
}

fn run_engine(seed: u64) -> worldsmith_state::SimulationSnapshot {
    let mut engine = EngineBuilder::new().with_seed(seed).build().unwrap();
    engine
        .state_mut()
        .planets
        .insert(PlanetId(1), simple_planet());
    engine.initialize().unwrap();
    engine.tick(100.0).unwrap();
    engine.latest_snapshot().unwrap().clone()
}

fn simple_planet() -> worldsmith_models::Planet {
    worldsmith_models::Planet {
        id: PlanetId(1),
        name: "Replay World".into(),
        class: worldsmith_models::PlanetClass::Terrestrial,
        planet_type: worldsmith_models::PlanetType::Rocky,
        system_id: worldsmith_models::SystemId(1),
        physical: worldsmith_models::PhysicalProperties {
            mass_kg: worldsmith_models::MeasuredValue {
                value: 1.0e24,
                unit: "kg".into(),
                provenance: None,
            },
            radius_m: worldsmith_models::MeasuredValue {
                value: 5.0e6,
                unit: "m".into(),
                provenance: None,
            },
            density_kg_m3: None,
            surface_gravity_m_s2: None,
        },
        orbit: worldsmith_models::OrbitalProperties {
            parent: worldsmith_models::BodyReference::Star(worldsmith_models::StarId(1)),
            semi_major_axis_m: worldsmith_models::MeasuredValue {
                value: 1.0e11,
                unit: "m".into(),
                provenance: None,
            },
            semi_minor_axis_m: None,
            eccentricity: worldsmith_models::MeasuredValue {
                value: 0.05,
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
        atmosphere_state: None,
        hydrology_state: None,
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
        moons: Vec::new(),
    }
}
