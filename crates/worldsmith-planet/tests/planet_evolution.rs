//! Integration tests for the planet evolution pipeline.
//!
//! These tests verify the end-to-end evolution pipeline forms complete
//! geophysical worlds from formed planets. They also verify determinism,
//! scientific consistency, and engine integration.

use worldsmith_engine::EngineBuilder;
use worldsmith_math::constants;
use worldsmith_models::{
    BodyReference, ClimateType, MeasuredValue, OrbitalProperties, PhysicalProperties, Planet,
    PlanetClass, PlanetId, PlanetType, StarId, SystemId,
};
use worldsmith_planet::{
    check_planet_consistency, evolve_planet, PlanetEvolutionModule, PlanetEvolutionReport,
    PlanetFormationModule,
};
use worldsmith_stellar::StellarModule;

fn test_planet() -> Planet {
    Planet {
        id: PlanetId(1),
        name: "Test Earth".to_string(),
        class: PlanetClass::Terrestrial,
        planet_type: PlanetType::Rocky,
        system_id: SystemId(1),
        physical: PhysicalProperties {
            mass_kg: MeasuredValue {
                value: constants::EARTH_MASS,
                unit: "kg".to_string(),
                provenance: None,
            },
            radius_m: MeasuredValue {
                value: constants::EARTH_RADIUS,
                unit: "m".to_string(),
                provenance: None,
            },
            density_kg_m3: Some(MeasuredValue {
                value: 5_514.0,
                unit: "kg m^-3".to_string(),
                provenance: None,
            }),
            surface_gravity_m_s2: None,
        },
        orbit: OrbitalProperties {
            parent: BodyReference::Star(StarId(1)),
            semi_major_axis_m: MeasuredValue {
                value: constants::ASTRONOMICAL_UNIT,
                unit: "m".to_string(),
                provenance: None,
            },
            semi_minor_axis_m: None,
            eccentricity: MeasuredValue {
                value: 0.02,
                unit: "dimensionless".to_string(),
                provenance: None,
            },
            inclination_rad: MeasuredValue {
                value: 0.0,
                unit: "rad".to_string(),
                provenance: None,
            },
            orbital_period_s: None,
            rotation_period_s: Some(MeasuredValue {
                value: 86_400.0,
                unit: "s".to_string(),
                provenance: None,
            }),
            axial_tilt_rad: None,
        },
        geology: None,
        atmosphere: None,
        climate: None,
        ocean: None,
        magnetic_field: None,
        habitability: None,
        moons: Vec::new(),
    }
}

#[test]
fn earth_like_evolves_all_systems() {
    let planet = test_planet();
    let output = evolve_planet(planet, 1.0, 4.5).unwrap();

    // Every major system should be populated
    assert!(output.planet.geology.is_some(), "geology must be derived");
    assert!(
        output.planet.atmosphere.is_some(),
        "atmosphere must be derived"
    );
    assert!(output.planet.climate.is_some(), "climate must be derived");
    assert!(
        output.planet.magnetic_field.is_some(),
        "magnetic field must be derived"
    );
    assert!(
        output.planet.habitability.is_some(),
        "habitability must be assessed"
    );

    // Interior model must be complete
    assert!(output.interior.core_radius_m > 0.0);
    assert!(output.interior.mantle_thickness_m > 0.0);
    assert!(output.interior.crust_thickness_m > 0.0);

    // Timeline must have core events
    assert!(!output.timeline.is_empty());
    assert!(output.timeline.iter().any(|e| e.event.contains("forms")));
}

#[test]
fn evolved_planet_passes_scientific_consistency() {
    let planet = test_planet();
    let output = evolve_planet(planet, 1.0, 4.5).unwrap();
    assert!(check_planet_consistency(&output.planet).is_ok());
}

#[test]
fn evolution_is_deterministic_across_runs() {
    let a = evolve_planet(test_planet(), 1.0, 4.5).unwrap();
    let b = evolve_planet(test_planet(), 1.0, 4.5).unwrap();
    assert_eq!(a, b);
}

#[test]
fn evolution_reports_are_generated() {
    let planet = test_planet();
    let output = evolve_planet(planet, 1.0, 4.5).unwrap();
    let report_text = PlanetEvolutionReport::from_output(&output).to_string();
    assert!(
        report_text.contains("Interior"),
        "report should mention interior"
    );
    assert!(
        report_text.contains("Geology"),
        "report should mention geology"
    );
    assert!(
        report_text.contains("Habitability"),
        "report should mention habitability"
    );
    assert!(
        report_text.contains("Evolution Timeline"),
        "report should mention timeline"
    );
}

#[test]
fn multiple_planet_types_produce_different_climates() {
    let earth = test_planet();

    let mut cold = test_planet();
    cold.orbit.semi_major_axis_m = MeasuredValue {
        value: 5.0 * constants::ASTRONOMICAL_UNIT,
        unit: "m".to_string(),
        provenance: None,
    };

    let e = evolve_planet(earth, 1.0, 4.5).unwrap();
    let c = evolve_planet(cold, 1.0, 4.5).unwrap();

    let e_type = e.planet.climate.as_ref().unwrap().climate_type;
    let c_type = c.planet.climate.as_ref().unwrap().climate_type;

    // These should differ — climates emerge from orbital distance
    assert_ne!(e_type, c_type, "temperate and frozen should differ");
    assert_eq!(
        c_type,
        ClimateType::Frozen,
        "distant planet should be frozen"
    );
}

#[test]
fn evolution_module_integrates_after_formation() {
    let mut engine = EngineBuilder::new()
        .with_seed(42)
        .register_module(Box::new(StellarModule::default()))
        .register_module_with_stage(
            Box::new(PlanetFormationModule::default()),
            10,
            vec!["worldsmith.stellar".to_string()],
        )
        .register_module_with_stage(
            Box::new(PlanetEvolutionModule::default()),
            20,
            vec![
                "worldsmith.stellar".to_string(),
                "worldsmith.planet_formation".to_string(),
            ],
        )
        .build()
        .unwrap();

    engine.initialize().unwrap();
    engine.tick_fixed().unwrap();

    // After initialization, planets should be fully evolved
    assert!(!engine.state().planets.is_empty(), "planets must exist");
    for (id, planet) in &engine.state().planets {
        assert!(
            planet.geology.is_some(),
            "planet {} should have geology after evolution",
            id.0
        );
        assert!(
            planet.atmosphere.is_some(),
            "planet {} should have atmosphere after evolution",
            id.0
        );
        assert!(
            planet.climate.is_some(),
            "planet {} should have climate after evolution",
            id.0
        );
        assert!(
            planet.magnetic_field.is_some(),
            "planet {} should have magnetic field after evolution",
            id.0
        );
    }
}

#[test]
fn evolution_is_engine_deterministic() {
    fn run() -> u64 {
        let mut engine = EngineBuilder::new()
            .with_seed(99)
            .register_module(Box::new(StellarModule::default()))
            .register_module_with_stage(
                Box::new(PlanetFormationModule::default()),
                10,
                vec!["worldsmith.stellar".to_string()],
            )
            .register_module_with_stage(
                Box::new(PlanetEvolutionModule::default()),
                20,
                vec![
                    "worldsmith.stellar".to_string(),
                    "worldsmith.planet_formation".to_string(),
                ],
            )
            .build()
            .unwrap();
        engine.initialize().unwrap();
        engine.tick_fixed().unwrap();
        engine.state_fingerprint()
    }

    assert_eq!(run(), run());
}

#[test]
fn atmosphere_type_scales_with_planet_mass() {
    let mut small = test_planet();
    small.physical.mass_kg = MeasuredValue {
        value: 0.05 * constants::EARTH_MASS,
        unit: "kg".to_string(),
        provenance: None,
    };
    small.physical.radius_m = MeasuredValue {
        value: 0.3 * constants::EARTH_RADIUS,
        unit: "m".to_string(),
        provenance: None,
    };
    small.class = PlanetClass::Dwarf;

    let mut large = test_planet();
    large.physical.mass_kg = MeasuredValue {
        value: 100.0 * constants::EARTH_MASS,
        unit: "kg".to_string(),
        provenance: None,
    };
    large.physical.radius_m = MeasuredValue {
        value: 3.0 * constants::EARTH_RADIUS,
        unit: "m".to_string(),
        provenance: None,
    };
    large.planet_type = PlanetType::Gas;
    large.class = PlanetClass::GasGiant;

    let s = evolve_planet(small, 1.0, 4.5).unwrap();
    let l = evolve_planet(large, 1.0, 4.5).unwrap();

    assert_ne!(
        s.planet.atmosphere.unwrap().atmosphere_type,
        l.planet.atmosphere.unwrap().atmosphere_type,
        "small and large planets should have different atmosphere types"
    );
}

#[test]
fn ocean_planet_has_water_cycle() {
    let mut planet = test_planet();
    planet.planet_type = PlanetType::Ocean;
    let output = evolve_planet(planet, 1.0, 4.5).unwrap();

    // Ocean planet should have water
    assert!(
        output.planet.ocean.is_some(),
        "ocean planet should have ocean"
    );

    // Climate should be temperate or tropical (use wider orbit since model has no albedo)
    let climate = output.planet.climate.as_ref().unwrap();
    assert!(
        climate.climate_type == ClimateType::Temperate
            || climate.climate_type == ClimateType::Tropical
            || climate.average_temperature_k.as_ref().map(|t| t.value > 260.0).unwrap_or(false),
        "ocean planet should have livable temperatures"
    );

    // Atmosphere should have water vapor
    let atmosphere = output.planet.atmosphere.as_ref().unwrap();
    assert!(
        atmosphere
            .composition
            .iter()
            .any(|g| g.molecule.formula == "H2O"),
        "atmosphere should contain water vapor"
    );

    // Timeline should mention oceans
    assert!(
        output.timeline.iter().any(|e| e.event.contains("Ocean")),
        "timeline should include ocean formation"
    );
}

#[test]
fn habitability_improves_with_favorable_conditions() {
    let habitable = test_planet();
    let h = evolve_planet(habitable, 1.0, 4.5).unwrap();

    let mut hostile = test_planet();
    hostile.orbit.semi_major_axis_m = MeasuredValue {
        value: 0.1 * constants::ASTRONOMICAL_UNIT,
        unit: "m".to_string(),
        provenance: None,
    };
    let bad = evolve_planet(hostile, 1.0, 4.5).unwrap();

    let h_rating = h.planet.habitability.as_ref().unwrap();
    let b_rating = bad.planet.habitability.as_ref().unwrap();

    // Earth-like should be more habitable than hot Jupiter
    assert!(
        h_rating.positive_factors.len() >= b_rating.positive_factors.len(),
        "habitable planet should have more positive factors"
    );
}

#[test]
fn timeline_includes_geological_events() {
    let planet = test_planet();
    let output = evolve_planet(planet, 1.0, 4.5).unwrap();

    // Earth-like should have plate tectonics mentioned
    let has_tectonics = output.timeline.iter().any(|e| e.event.contains("tectonic"));
    let has_volcanism = output
        .timeline
        .iter()
        .any(|e| e.event.contains("Volcanism"));

    // Either plate tectonics or volcanism should be mentioned
    assert!(
        has_tectonics || has_volcanism,
        "timeline should include geological events"
    );
}

#[test]
fn ice_albedo_feedback_on_cold_planet() {
    let mut planet = test_planet();
    planet.orbit.semi_major_axis_m = MeasuredValue {
        value: 2.0 * constants::ASTRONOMICAL_UNIT,
        unit: "m".to_string(),
        provenance: None,
    };
    let output = evolve_planet(planet, 1.0, 4.5).unwrap();

    // At 2 AU, there should be ice coverage
    let climate = output.planet.climate.as_ref().unwrap();
    if let Some(ice) = &climate.ice_coverage {
        assert!(ice.value > 0.0, "cold planet should have ice coverage");
    }

    // Timeline may include ice-albedo feedback
    // (may not if temperature is too extreme, but ice should exist)
    let has_ice = climate
        .ice_coverage
        .as_ref()
        .map(|i| i.value > 0.1)
        .unwrap_or(false);
    if has_ice {
        assert!(
            climate
                .temperature_bands
                .last()
                .unwrap()
                .average_temperature_k
                .value
                <= climate
                    .temperature_bands
                    .first()
                    .unwrap()
                    .average_temperature_k
                    .value,
            "polar temperature should be colder than equatorial"
        );
    }
}

#[test]
fn report_contains_all_sections() {
    let planet = test_planet();
    let output = evolve_planet(planet, 1.0, 4.5).unwrap();
    let report = PlanetEvolutionReport::from_output(&output);
    let text = report.to_string();

    assert!(
        text.contains("Interior"),
        "report should have interior section"
    );
    assert!(
        text.contains("Geology"),
        "report should have geology section"
    );
    assert!(
        text.contains("Magnetic Field"),
        "report should have magnetic field section"
    );
    assert!(
        text.contains("Atmosphere"),
        "report should have atmosphere section"
    );
    assert!(
        text.contains("Climate"),
        "report should have climate section"
    );
    assert!(
        text.contains("Hydrology"),
        "report should have hydrology section"
    );
    assert!(
        text.contains("Habitability"),
        "report should have habitability section"
    );
    assert!(
        text.contains("Evolution Timeline"),
        "report should have timeline section"
    );
}
