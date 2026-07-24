//! State validation tests.

use worldsmith_engine::EngineBuilder;
use worldsmith_evolution::{
    CoreEvolutionModule, MantleEvolutionModule, PlateTectonicsModule, VolcanismModule,
};
use worldsmith_math::Vector3;
use worldsmith_models::{
    BodyReference, MeasuredValue, OrbitalProperties, PhysicalProperties, Planet, PlanetId,
    PlanetType, StarId, SystemId,
};
use worldsmith_stellar::StellarModule;
use worldsmith_validation::{validate_state, StateValidationError};

fn earth_like_planet() -> Planet {
    Planet {
        id: PlanetId(1),
        name: "Earth".into(),
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

#[test]
fn validate_state_passes_on_empty_engine() {
    let mut engine = EngineBuilder::new()
        .with_seed(7)
        .register_module(Box::new(StellarModule::default()))
        .register_module(Box::new(CoreEvolutionModule::default()))
        .register_module(Box::new(MantleEvolutionModule::default()))
        .register_module(Box::new(VolcanismModule::default()))
        .register_module(Box::new(PlateTectonicsModule::default()))
        .build()
        .unwrap();
    engine.initialize().unwrap();
    assert!(validate_state(engine.state()).is_ok());
}

#[test]
fn validate_state_passes_after_tick() {
    let mut engine = EngineBuilder::new()
        .with_seed(7)
        .register_module(Box::new(StellarModule::default()))
        .register_module(Box::new(CoreEvolutionModule::default()))
        .register_module(Box::new(MantleEvolutionModule::default()))
        .register_module(Box::new(VolcanismModule::default()))
        .register_module(Box::new(PlateTectonicsModule::default()))
        .build()
        .unwrap();
    engine
        .state_mut()
        .planets
        .insert(PlanetId(1), earth_like_planet());
    engine.initialize().unwrap();
    engine.tick_fixed().unwrap();
    assert!(validate_state(engine.state()).is_ok());
}

#[test]
fn validate_state_detects_nan_in_core_temperature() {
    let mut engine = EngineBuilder::new()
        .with_seed(7)
        .register_module(Box::new(StellarModule::default()))
        .register_module(Box::new(CoreEvolutionModule::default()))
        .register_module(Box::new(MantleEvolutionModule::default()))
        .build()
        .unwrap();
    engine
        .state_mut()
        .planets
        .insert(PlanetId(1), earth_like_planet());
    engine.initialize().unwrap();
    engine.tick_fixed().unwrap();

    if let Some(planet) = engine.state_mut().planets.get_mut(&PlanetId(1)) {
        if let Some(interior) = planet.interior.as_mut() {
            interior.core_temperature = f64::NAN;
        }
    }

    let result = validate_state(engine.state());
    assert!(result.is_err());
    match result.unwrap_err() {
        StateValidationError::Nan { field, .. } => assert_eq!(field, "core_temperature"),
        _ => panic!("expected Nan error"),
    }
}

#[test]
fn validate_state_detects_infinite_mantle_temperature() {
    let mut engine = EngineBuilder::new()
        .with_seed(7)
        .register_module(Box::new(StellarModule::default()))
        .register_module(Box::new(CoreEvolutionModule::default()))
        .register_module(Box::new(MantleEvolutionModule::default()))
        .build()
        .unwrap();
    engine
        .state_mut()
        .planets
        .insert(PlanetId(1), earth_like_planet());
    engine.initialize().unwrap();
    engine.tick_fixed().unwrap();

    if let Some(planet) = engine.state_mut().planets.get_mut(&PlanetId(1)) {
        if let Some(interior) = planet.interior.as_mut() {
            interior.mantle_temperature = f64::INFINITY;
        }
    }

    let result = validate_state(engine.state());
    assert!(result.is_err());
    match result.unwrap_err() {
        StateValidationError::Infinity { field, .. } => assert_eq!(field, "mantle_temperature"),
        _ => panic!("expected Infinity error"),
    }
}
