//! Long-term stability tests for the Phase 10 evolution subsystem.

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
use worldsmith_validation::run_long_run_stability;

fn build_stability_engine() -> worldsmith_engine::Engine {
    let mut engine = EngineBuilder::new()
        .with_seed(7)
        .register_module(Box::new(StellarModule::default()))
        .register_module(Box::new(CoreEvolutionModule::default()))
        .register_module(Box::new(MantleEvolutionModule::default()))
        .register_module(Box::new(VolcanismModule::default()))
        .register_module(Box::new(PlateTectonicsModule::default()))
        .build()
        .unwrap();

    let planet = Planet {
        id: PlanetId(1),
        name: "Stability".into(),
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
    };
    engine.state_mut().planets.insert(PlanetId(1), planet);
    engine
}

#[test]
fn stability_100_ticks() {
    let engine = build_stability_engine();
    let report = run_long_run_stability(engine, 100).unwrap();
    assert!(report.stable);
    assert_eq!(report.ticks, 100);
    assert!(report.state_errors.is_empty());
    assert!(report.max_abs_value.is_finite());
}

#[test]
fn stability_1_000_ticks() {
    let engine = build_stability_engine();
    let report = run_long_run_stability(engine, 1_000).unwrap();
    assert!(report.stable);
    assert_eq!(report.ticks, 1_000);
    assert!(report.state_errors.is_empty());
    assert!(report.max_abs_value.is_finite());
}

#[test]
fn stability_10_000_ticks() {
    let engine = build_stability_engine();
    let report = run_long_run_stability(engine, 10_000).unwrap();
    assert!(report.stable);
    assert_eq!(report.ticks, 10_000);
    assert!(report.state_errors.is_empty());
    assert!(report.max_abs_value.is_finite());
}

#[test]
fn stability_100_000_ticks() {
    let engine = build_stability_engine();
    let report = run_long_run_stability(engine, 100_000).unwrap();
    assert!(report.stable);
    assert_eq!(report.ticks, 100_000);
    assert!(report.state_errors.is_empty());
    assert!(report.max_abs_value.is_finite());
}
