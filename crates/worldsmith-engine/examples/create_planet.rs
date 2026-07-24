//! Create a basic planet and initialize it in the engine.

use worldsmith_engine::EngineBuilder;
use worldsmith_evolution::EvolutionPlugin;
use worldsmith_math::Vector3;
use worldsmith_models::{
    BodyReference, MeasuredValue, OrbitalProperties, PhysicalProperties, Planet, PlanetId,
    PlanetType, StarId, SystemId,
};

fn main() -> worldsmith_engine::EngineResult<()> {
    let mut engine = EngineBuilder::new().with_seed(12345).build()?;

    let planet = Planet {
        id: PlanetId(1),
        name: "Terra Nova".into(),
        class: worldsmith_models::PlanetClass::Terrestrial,
        planet_type: PlanetType::Rocky,
        system_id: SystemId(1),
        physical: PhysicalProperties {
            mass_kg: MeasuredValue {
                value: 5.97e24,
                unit: "kg".into(),
                provenance: None,
            },
            radius_m: MeasuredValue {
                value: 6.37e6,
                unit: "m".into(),
                provenance: None,
            },
            density_kg_m3: None,
            surface_gravity_m_s2: None,
        },
        orbit: OrbitalProperties {
            parent: BodyReference::Star(StarId(1)),
            semi_major_axis_m: MeasuredValue {
                value: 1.5e11,
                unit: "m".into(),
                provenance: None,
            },
            semi_minor_axis_m: None,
            eccentricity: MeasuredValue {
                value: 0.02,
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
        position_m: Vector3::ZERO,
        velocity_m_s: Vector3::ZERO,
        moons: Vec::new(),
    };

    engine.state_mut().planets.insert(PlanetId(1), planet);
    engine.initialize()?;

    println!(
        "Created planet: {:?}",
        engine.state().planets.get(&PlanetId(1)).unwrap().name
    );
    Ok(())
}
