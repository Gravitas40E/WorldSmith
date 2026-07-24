//! Inspect a planet's state after evolution.

use worldsmith_engine::EngineBuilder;
use worldsmith_evolution::EvolutionPlugin;
use worldsmith_models::PlanetId;

fn main() -> worldsmith_engine::EngineResult<()> {
    let mut engine = EngineBuilder::new().with_seed(99).build()?;

    engine
        .state_mut()
        .planets
        .insert(PlanetId(1), simple_planet());
    engine.initialize()?;
    engine.tick(200.0)?;

    let planet = engine.state().planets.get(&PlanetId(1)).unwrap();
    println!("Planet: {}", planet.name);
    if let Some(atmo) = &planet.atmosphere_state {
        println!("  Surface pressure: {:.2} Pa", atmo.surface_pressure_pa);
    }
    if let Some(hydro) = &planet.hydrology_state {
        println!(
            "  Liquid water: {:.2}%",
            hydro.liquid_water_fraction * 100.0
        );
    }
    if let Some(climate) = &planet.climate_state {
        println!(
            "  Equilibrium temp: {:.2} K",
            climate.equilibrium_temperature_k
        );
    }
    if let Some(class) = &planet.classification_state {
        println!(
            "  Classification: {:?} {:?}",
            class.primary_classification, class.secondary_classification
        );
        println!("  Confidence: {:.2}", class.confidence_score);
        println!("  Summary: {}", class.classification_summary);
    }
    Ok(())
}

fn simple_planet() -> worldsmith_models::Planet {
    worldsmith_models::Planet {
        id: PlanetId(1),
        name: "Inspected World".into(),
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
