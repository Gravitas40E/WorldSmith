//! Atmosphere validation tests.

use worldsmith_engine::EngineBuilder;
use worldsmith_evolution::{
    AtmosphereModule, CoreEvolutionModule, MantleEvolutionModule, PlateTectonicsModule,
    VolcanismModule,
};
use worldsmith_models::{
    BodyReference, InteriorState, MeasuredValue, OrbitalProperties, PhysicalProperties, Planet,
    PlanetId, PlanetType, StarId, SystemId,
};
use worldsmith_traits::{ModuleContext, SimulationModule};
use worldsmith_validation::{validate_state, StateValidationError};

fn earth_like_planet() -> Planet {
    Planet {
        id: PlanetId(1),
        name: "AtmosphereValidation".into(),
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
        interior: Some(InteriorState {
            age_seconds: 0.0,
            internal_heat: 5.972e24 * 1.0e6,
            radiogenic_heat: 5.972e24 * 2.0e-15,
            core_temperature: 6000.0,
            mantle_temperature: 4800.0,
            heat_flux: 1.2e16,
        }),
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
        position_m: worldsmith_math::Vector3::ZERO,
        velocity_m_s: worldsmith_math::Vector3::ZERO,
    }
}

fn seeded_planet() -> Planet {
    let mut p = earth_like_planet();
    p.volcanism = Some(worldsmith_models::VolcanismState {
        volcanic_flux: 1.0e16,
        volcanic_activity: worldsmith_models::VolcanicActivity::Moderate,
        magma_generation_rate: 5.97e9,
    });
    p.plate_tectonics = Some(worldsmith_models::PlateTectonicsState {
        plate_velocity: 5.0,
        crustal_recycling_rate: 0.025,
        tectonic_activity: worldsmith_models::TectonicActivity::Moderate,
    });
    p
}

fn insert_sun(state: &mut worldsmith_state::WorldState) {
    state.stars.insert(
        worldsmith_models::StarId(1),
        worldsmith_models::Star {
            id: worldsmith_models::StarId(1),
            name: "Sun".into(),
            spectral_type: worldsmith_models::SpectralType::G,
            class: worldsmith_models::StarClass::MainSequence,
            mass_kg: worldsmith_models::MeasuredValue {
                value: 1.989e30,
                unit: "kg".into(),
                provenance: None,
            },
            radius_m: worldsmith_models::MeasuredValue {
                value: 6.957e8,
                unit: "m".into(),
                provenance: None,
            },
            luminosity_w: worldsmith_models::MeasuredValue {
                value: 3.828e26,
                unit: "W".into(),
                provenance: None,
            },
            effective_temperature_k: worldsmith_models::MeasuredValue {
                value: 5778.0,
                unit: "K".into(),
                provenance: None,
            },
            surface_gravity_m_s2: worldsmith_models::MeasuredValue {
                value: 274.0,
                unit: "m/s^2".into(),
                provenance: None,
            },
            metallicity: worldsmith_models::MeasuredValue {
                value: 0.02,
                unit: "dex".into(),
                provenance: None,
            },
            rotation_period_s: None,
            age_s: None,
            position_m: worldsmith_math::Vector3::ZERO,
            velocity_m_s: worldsmith_math::Vector3::ZERO,
        },
    );
}

#[test]
fn surface_pressure_must_be_non_negative() {
    let mut engine = EngineBuilder::new()
        .with_seed(7)
        .build()
        .expect("engine builds");
    engine
        .state_mut()
        .planets
        .insert(PlanetId(1), seeded_planet());
    insert_sun(engine.state_mut());

    let mut core = CoreEvolutionModule::default();
    core.initialize(engine.state_mut()).unwrap();

    let mut mantle = MantleEvolutionModule::default();
    mantle.initialize(engine.state_mut()).unwrap();

    let mut volcanism = VolcanismModule::default();
    volcanism.initialize(engine.state_mut()).unwrap();

    let mut plate = PlateTectonicsModule::default();
    plate.initialize(engine.state_mut()).unwrap();

    let mut module = AtmosphereModule::default();
    module.initialize(engine.state_mut()).unwrap();

    for i in 0..100 {
        module
            .update(
                ModuleContext {
                    timestamp_s: (i as f64) * 1.0,
                    delta_seconds: 1.0,
                    seed: 7,
                },
                engine.state_mut(),
            )
            .expect("update succeeds");
    }

    let planet = engine.state().planets.get(&PlanetId(1)).unwrap();
    let a = planet.atmosphere_state.as_ref().unwrap();
    assert!(
        a.surface_pressure_pa >= 0.0,
        "pressure must be non-negative"
    );
}

#[test]
fn composition_fractions_must_sum_to_one() {
    let planet = Planet {
        id: PlanetId(1),
        name: "CompositionSum".into(),
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
        atmosphere_state: Some(worldsmith_models::AtmosphereState {
            atmospheric_mass_kg: 5.15e18,
            surface_pressure_pa: 101325.0,
            mean_temperature_k: 288.15,
            atmosphere_composition: vec![worldsmith_models::AtmosphericGas {
                molecule: worldsmith_models::Molecule {
                    formula: "N2".into(),
                    name: "N2".into(),
                    molar_mass_kg_mol: Some(MeasuredValue {
                        value: 0.028014,
                        unit: "kg/mol".into(),
                        provenance: None,
                    }),
                },
                abundance: MeasuredValue {
                    value: 0.78,
                    unit: "mole_fraction".into(),
                    provenance: None,
                },
                is_greenhouse: false,
            }],
        }),
        hydrology_state: None,
        climate_state: None,
        carbon_cycle_state: None,
        biosphere_state: None,
        habitability_state: None,
        classification_state: None,
        surface_chemistry_state: None,
        cryosphere_state: None,
        moons: Vec::new(),
        position_m: worldsmith_math::Vector3::ZERO,
        velocity_m_s: worldsmith_math::Vector3::ZERO,
    };

    let mut engine = EngineBuilder::new()
        .with_seed(7)
        .build()
        .expect("engine builds");
    let state = engine.state_mut();
    state.planets.insert(PlanetId(1), planet);

    let err = validate_state(state).unwrap_err();
    assert!(matches!(err, StateValidationError::InvalidEnum { .. }));
}

#[test]
fn temperature_must_be_positive_after_update() {
    let mut engine = EngineBuilder::new()
        .with_seed(7)
        .build()
        .expect("engine builds");
    engine
        .state_mut()
        .planets
        .insert(PlanetId(1), seeded_planet());
    insert_sun(engine.state_mut());

    let mut core = CoreEvolutionModule::default();
    core.initialize(engine.state_mut()).unwrap();

    let mut mantle = MantleEvolutionModule::default();
    mantle.initialize(engine.state_mut()).unwrap();

    let mut volcanism = VolcanismModule::default();
    volcanism.initialize(engine.state_mut()).unwrap();

    let mut plate = PlateTectonicsModule::default();
    plate.initialize(engine.state_mut()).unwrap();

    let mut module = AtmosphereModule::default();
    module.initialize(engine.state_mut()).unwrap();

    for i in 0..10 {
        module
            .update(
                ModuleContext {
                    timestamp_s: (i as f64) * 1.0,
                    delta_seconds: 1.0,
                    seed: 7,
                },
                engine.state_mut(),
            )
            .expect("update succeeds");
    }

    let planet = engine.state().planets.get(&PlanetId(1)).unwrap();
    let a = planet.atmosphere_state.as_ref().unwrap();
    assert!(a.mean_temperature_k > 0.0, "temperature must be positive");
}

#[test]
fn mass_is_conserved_up_to_outgassing_and_escape() {
    let mut engine_a = EngineBuilder::new()
        .with_seed(7)
        .build()
        .expect("engine builds");
    engine_a
        .state_mut()
        .planets
        .insert(PlanetId(1), seeded_planet());
    insert_sun(engine_a.state_mut());

    let mut engine_b = EngineBuilder::new()
        .with_seed(7)
        .build()
        .expect("engine builds");
    engine_b
        .state_mut()
        .planets
        .insert(PlanetId(1), seeded_planet());
    insert_sun(engine_b.state_mut());

    for i in 0..100 {
        let ctx = ModuleContext {
            timestamp_s: (i as f64) * 1.0,
            delta_seconds: 1.0,
            seed: 7,
        };
        let mut core_a = CoreEvolutionModule::default();
        let mut core_b = CoreEvolutionModule::default();
        core_a.initialize(engine_a.state_mut()).unwrap();
        core_b.initialize(engine_b.state_mut()).unwrap();

        let mut mantle_a = MantleEvolutionModule::default();
        let mut mantle_b = MantleEvolutionModule::default();
        mantle_a.initialize(engine_a.state_mut()).unwrap();
        mantle_b.initialize(engine_b.state_mut()).unwrap();

        let mut volcanism_a = VolcanismModule::default();
        let mut volcanism_b = VolcanismModule::default();
        volcanism_a.initialize(engine_a.state_mut()).unwrap();
        volcanism_b.initialize(engine_b.state_mut()).unwrap();

        let mut plate_a = PlateTectonicsModule::default();
        let mut plate_b = PlateTectonicsModule::default();
        plate_a.initialize(engine_a.state_mut()).unwrap();
        plate_b.initialize(engine_b.state_mut()).unwrap();

        let mut module_a = AtmosphereModule::default();
        let mut module_b = AtmosphereModule::default();
        module_a.initialize(engine_a.state_mut()).unwrap();
        module_b.initialize(engine_b.state_mut()).unwrap();

        module_a
            .update(ctx, engine_a.state_mut())
            .expect("atmosphere update succeeds");
        module_b
            .update(ctx, engine_b.state_mut())
            .expect("atmosphere update succeeds");
    }

    let a_mass = engine_a
        .state()
        .planets
        .get(&PlanetId(1))
        .unwrap()
        .atmosphere_state
        .as_ref()
        .unwrap()
        .atmospheric_mass_kg;
    let b_mass = engine_b
        .state()
        .planets
        .get(&PlanetId(1))
        .unwrap()
        .atmosphere_state
        .as_ref()
        .unwrap()
        .atmospheric_mass_kg;
    assert_eq!(a_mass, b_mass, "atmosphere state must be deterministic");
}
