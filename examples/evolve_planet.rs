//! Evolve a planet through 100 ticks using the default evolution plugin.

use worldsmith_engine::EngineBuilder;
use worldsmith_evolution::EvolutionPlugin;
use worldsmith_math::Vector3;
use worldsmith_models::{
    BodyReference, MeasuredValue, OrbitalProperties, PhysicalProperties, Planet, PlanetId,
    PlanetType, StarId, SystemId,
};

fn main() -> worldsmith_engine::EngineResult<()> {
    let mut engine = EngineBuilder::new()
        .with_seed(42)
        .register_module_with_stage(
            Box::new(worldsmith_evolution::PlanetClassificationModule::default()),
            -1,
            vec!["worldsmith.evolution.habitability".to_string()],
        )
        .build()?;

    let planet = earth_like_planet();
    engine.state_mut().planets.insert(PlanetId(1), planet);
    engine.initialize()?;

    for tick in 0..100 {
        engine.tick(100.0)?;
        if (tick + 1) % 25 == 0 {
            let snap = engine.latest_snapshot();
            if let Some(s) = snap {
                if let Some(p) = s.planets.iter().find(|p| p.id == PlanetId(1)) {
                    let atmo = p.planet.atmosphere_state.as_ref();
                    let hydro = p.planet.hydrology_state.as_ref();
                    let climate = p.planet.climate_state.as_ref();
                    println!(
                        "Tick {}: atmo={:?} hydro={:?} climate={:?}",
                        tick + 1,
                        atmo.map(|a| a.surface_pressure_pa),
                        hydro.map(|h| h.liquid_water_fraction),
                        climate.map(|c| c.equilibrium_temperature_k)
                    );
                }
            }
        }
    }

    Ok(())
}

fn earth_like_planet() -> Planet {
    Planet {
        id: PlanetId(1),
        name: "Earth-like".into(),
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
    }
}
