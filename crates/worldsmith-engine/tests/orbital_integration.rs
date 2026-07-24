//! Integration tests for the orbital dynamics pipeline.
//!
//! These tests verify that `OrbitalDynamicsModule` integrates correctly with
//! the engine, produces propagated positions in snapshots, and that
//! visualization consumes those positions without performing orbital math.

use worldsmith_engine::EngineBuilder;
use worldsmith_math::Vector3;
use worldsmith_models::{
    BodyReference, MeasuredValue, OrbitalProperties, PhysicalProperties, Planet, PlanetId,
    PlanetType, StarId, SystemId,
};
use worldsmith_stellar::orbital_module::OrbitalDynamicsModule;
use worldsmith_stellar::StellarModule;
use worldsmith_visualization::bridge::{DefaultSnapshotBridge, SnapshotBridge};

const SOLAR_MASS: f64 = 1.989e30;
const AU: f64 = 1.496e11;

fn earth_like_planet(planet_id: PlanetId, star_id: StarId) -> Planet {
    Planet {
        id: planet_id,
        name: format!("Planet {}", planet_id.0),
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
            parent: BodyReference::Star(star_id),
            semi_major_axis_m: MeasuredValue {
                value: AU,
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
    }
}

#[test]
fn orbital_module_is_registered_and_executed() {
    let mut engine = EngineBuilder::new()
        .with_seed(42)
        .register_module(Box::new(StellarModule::default()))
        .register_module_with_stage(
            Box::new(OrbitalDynamicsModule::default()),
            20,
            vec!["worldsmith.stellar".to_string()],
        )
        .build()
        .unwrap();

    engine.initialize().unwrap();
    engine.tick_fixed().unwrap();

    let diagnostics = engine.diagnostics();
    assert!(
        diagnostics
            .registered_modules
            .contains(&"worldsmith.orbital".to_string()),
        "OrbitalDynamicsModule must be registered"
    );
    assert!(
        diagnostics
            .active_pipeline
            .contains(&"worldsmith.orbital".to_string()),
        "OrbitalDynamicsModule must appear in the active pipeline"
    );
}

#[test]
fn snapshot_contains_propagated_positions() {
    let mut engine = EngineBuilder::new()
        .with_seed(1)
        .register_module(Box::new(StellarModule::default()))
        .register_module_with_stage(
            Box::new(OrbitalDynamicsModule::default()),
            20,
            vec!["worldsmith.stellar".to_string()],
        )
        .build()
        .unwrap();

    engine.initialize().unwrap();

    // Inject a planet directly into world state before ticking.
    let planet = earth_like_planet(PlanetId(1), StarId(1));
    engine.state_mut().planets.insert(PlanetId(1), planet);

    engine.tick_fixed().unwrap();

    let snapshot = engine.latest_snapshot().expect("snapshot must exist");
    assert_eq!(snapshot.planets.len(), 1);
    let propagated = &snapshot.planets[0].planet;

    // After one tick, the planet should have moved away from ZERO.
    let dist = (propagated.position_m.x.powi(2)
        + propagated.position_m.y.powi(2)
        + propagated.position_m.z.powi(2))
    .sqrt();
    assert!(
        dist > 1.0,
        "planet should have non-zero propagated position, got {}",
        dist
    );
}

#[test]
fn visualization_consumes_snapshot_coordinates() {
    let bridge = DefaultSnapshotBridge;

    let snapshot = worldsmith_state::SimulationSnapshot {
        metadata: worldsmith_state::SimulationMetadata::default(),
        timestamp_s: 1.0,
        stellar: worldsmith_state::StellarSnapshot {
            systems: Vec::new(),
            stars: Vec::new(),
        },
        planets: vec![worldsmith_state::PlanetSnapshot {
            id: PlanetId(1),
            name: "Earth".into(),
            planet: Planet {
                id: PlanetId(1),
                name: "Earth".into(),
                class: worldsmith_models::PlanetClass::Terrestrial,
                planet_type: worldsmith_models::PlanetType::Rocky,
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
                        value: AU,
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
                position_m: Vector3::new(1.5e11, 0.0, 0.0),
                velocity_m_s: Vector3::new(0.0, 30_000.0, 0.0),
            },
        }],
        moons: Vec::new(),
    };

    let scene = bridge.build_scene(&snapshot);
    assert_eq!(scene.bodies.len(), 1);
    let body = &scene.bodies[0];
    assert_eq!(
        body.category,
        worldsmith_visualization::scene::BodyCategory::Planet
    );
    assert_eq!(body.position_m, [1.5e11, 0.0, 0.0]);
    assert_eq!(body.id, "1");
}

#[test]
fn no_visualization_side_orbital_projection_remains() {
    let bridge = DefaultSnapshotBridge;

    let snapshot = worldsmith_state::SimulationSnapshot {
        metadata: worldsmith_state::SimulationMetadata::default(),
        timestamp_s: 0.0,
        stellar: worldsmith_state::StellarSnapshot {
            systems: Vec::new(),
            stars: vec![worldsmith_models::Star {
                id: worldsmith_models::StarId(1),
                name: "Sun".into(),
                spectral_type: worldsmith_models::SpectralType::G,
                class: worldsmith_models::StarClass::MainSequence,
                mass_kg: MeasuredValue {
                    value: SOLAR_MASS,
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
            }],
        },
        planets: vec![worldsmith_state::PlanetSnapshot {
            id: PlanetId(1),
            name: "Earth".into(),
            planet: Planet {
                id: PlanetId(1),
                name: "Earth".into(),
                class: worldsmith_models::PlanetClass::Terrestrial,
                planet_type: worldsmith_models::PlanetType::Rocky,
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
                        value: AU,
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
                position_m: Vector3::new(1.496e11, 0.0, 0.0),
                velocity_m_s: Vector3::new(0.0, 29_780.0, 0.0),
            },
        }],
        moons: Vec::new(),
    };

    let scene = bridge.build_scene(&snapshot);
    let planet_body = scene
        .bodies
        .iter()
        .find(|b| b.category == worldsmith_visualization::scene::BodyCategory::Planet)
        .expect("planet body must exist");

    // The bridge must use the snapshot's propagated position directly.
    // If it ever recomputed from orbital elements, this would not match
    // the known propagated value.
    assert_eq!(planet_body.position_m, [1.496e11, 0.0, 0.0]);
}
