use worldsmith_engine::EngineBuilder;
use worldsmith_math::constants;
use worldsmith_planet::{
    accrete_planetesimals, available_materials, classify_embryo, disk_temperature_k,
    generate_planetesimals, surface_density_kg_m2, PlanetFormationBuilder, PlanetFormationModule,
    PlanetFormationReport, ProtoplanetaryDisk,
};
use worldsmith_stellar::StellarModule;

#[test]
fn disk_profiles_decrease_with_distance() {
    let disk = ProtoplanetaryDisk::from_star(1.0, 1.0, 0.0134, 1.0);
    let inner = constants::ASTRONOMICAL_UNIT;
    let outer = 5.0 * constants::ASTRONOMICAL_UNIT;

    assert!(disk_temperature_k(1.0, inner, 1.0) > disk_temperature_k(1.0, outer, 1.0));
    assert!(
        surface_density_kg_m2(disk.disk_mass_kg, disk.disk_radius_m, inner, 1.5)
            > surface_density_kg_m2(disk.disk_mass_kg, disk.disk_radius_m, outer, 1.5)
    );
}

#[test]
fn condensation_sequence_adds_ices_when_cold() {
    let hot = available_materials(1_000.0);
    let cold = available_materials(120.0);
    assert!(hot.len() < cold.len());
}

#[test]
fn planetesimal_generation_is_deterministic() {
    let disk = ProtoplanetaryDisk::from_star(1.0, 1.0, 0.0134, 1.0);
    let mut a = worldsmith_rng::RngStream::new(7);
    let mut b = worldsmith_rng::RngStream::new(7);
    assert_eq!(
        generate_planetesimals(&disk, 16, &mut a),
        generate_planetesimals(&disk, 16, &mut b)
    );
}

#[test]
fn accretion_promotes_embryos_and_classifies_planets() {
    let disk = ProtoplanetaryDisk::from_star(1.0, 1.0, 0.0134, 1.0);
    let mut rng = worldsmith_rng::RngStream::new(9);
    let bodies = generate_planetesimals(&disk, 64, &mut rng);
    let summary = accrete_planetesimals(bodies, 1.0e21);
    assert!(!summary.embryos.is_empty());
    let (class, _kind) = classify_embryo(&summary.embryos[0]);
    assert!(matches!(
        class,
        worldsmith_models::PlanetClass::Dwarf
            | worldsmith_models::PlanetClass::Terrestrial
            | worldsmith_models::PlanetClass::SuperEarth
            | worldsmith_models::PlanetClass::MiniNeptune
            | worldsmith_models::PlanetClass::IceGiant
            | worldsmith_models::PlanetClass::GasGiant
    ));
}

#[test]
fn formation_builder_and_report_are_deterministic() {
    let disk = ProtoplanetaryDisk::from_star(1.0, 1.0, 0.0134, 1.0);
    let a = PlanetFormationBuilder::new()
        .seed(42)
        .disk(disk.clone())
        .build()
        .unwrap();
    let b = PlanetFormationBuilder::new()
        .seed(42)
        .disk(disk)
        .build()
        .unwrap();
    assert_eq!(a, b);
    let report = PlanetFormationReport::from_result(&a, 0)
        .unwrap()
        .to_string();
    assert!(report.contains("Planet Report"));
    assert!(report.contains("Formation History"));
}

#[test]
fn planet_module_integrates_after_stellar_module() {
    let mut engine = EngineBuilder::new()
        .with_seed(123)
        .register_module(Box::new(StellarModule::default()))
        .register_module_with_stage(
            Box::new(PlanetFormationModule::default()),
            10,
            vec!["worldsmith.stellar".to_string()],
        )
        .build()
        .unwrap();

    engine.initialize().unwrap();
    engine.tick_fixed().unwrap();
    assert!(!engine.state().stars.is_empty());
    assert!(!engine.state().planets.is_empty());
    assert_eq!(engine.diagnostics().queued_event_count, 0);
}

#[test]
fn planet_engine_runs_are_repeatable() {
    fn run() -> u64 {
        let mut engine = EngineBuilder::new()
            .with_seed(55)
            .register_module(Box::new(StellarModule::default()))
            .register_module_with_stage(
                Box::new(PlanetFormationModule::default()),
                10,
                vec!["worldsmith.stellar".to_string()],
            )
            .build()
            .unwrap();
        engine.initialize().unwrap();
        engine.tick_fixed().unwrap();
        engine.state_fingerprint()
    }

    assert_eq!(run(), run());
}
