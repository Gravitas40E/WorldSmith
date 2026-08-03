//! Create a simulation snapshot and print it.

use worldsmith_engine::EngineBuilder;
use worldsmith_models::{Planet, PlanetId};

fn main() -> worldsmith_engine::EngineResult<()> {
    let mut engine = EngineBuilder::new().with_seed(7).build()?;

    engine
        .state_mut()
        .planets
        .insert(PlanetId(1), simple_planet());
    engine.initialize()?;
    engine.tick(50.0)?;

    let snap = engine.latest_snapshot().expect("snapshot");
    println!("Snapshot id: {}", snap.metadata.simulation_id);
    println!("Planets: {}", snap.planets.len());
    for p in &snap.planets {
        println!(
            "  {} — classification: {:?}",
            p.planet.name, p.planet.classification_state
        );
    }

    Ok(())
}

fn simple_planet() -> Planet {
    worldsmith_models::Planet {
        id: PlanetId(1),
        name: "Snap World".into(),
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
