use worldsmith_engine::EngineBuilder;
use worldsmith_stellar::{
    classification::{classify_star, LuminosityClass},
    frost_lines, habitable_zone, main_sequence_lifetime_gyr, StarBuilder, StellarModule,
    StellarModuleConfig, StellarReport,
};

#[test]
fn habitable_zone_and_frost_line_match_solar_references() {
    let hz = habitable_zone(1.0);
    let frost = frost_lines(1.0);
    assert!((hz.optimistic_inner_au - 0.75).abs() < 0.02);
    assert!((hz.conservative_outer_au - 1.67).abs() < 0.02);
    assert!((frost.water_au - 2.7).abs() < 0.1);
}

#[test]
fn classification_is_temperature_driven() {
    let g = classify_star(5_772.0, LuminosityClass::MainSequence);
    let m = classify_star(3_200.0, LuminosityClass::MainSequence);
    assert!(g.notation.starts_with('G'));
    assert!(m.notation.starts_with('M'));
}

#[test]
fn builder_is_deterministic() {
    let a = StarBuilder::new()
        .name("Sol")
        .mass_solar(1.0)
        .age_gyr(4.57)
        .metallicity(0.0134)
        .rotation_days(25.4)
        .build()
        .unwrap();
    let b = StarBuilder::new()
        .name("Sol")
        .mass_solar(1.0)
        .age_gyr(4.57)
        .metallicity(0.0134)
        .rotation_days(25.4)
        .build()
        .unwrap();
    assert_eq!(a, b);
    assert!((main_sequence_lifetime_gyr(1.0) - 10.0).abs() < 1e-12);
}

#[test]
fn report_contains_solar_reference_values() {
    let profile = StarBuilder::new()
        .name("Sol")
        .mass_solar(1.0)
        .age_gyr(4.57)
        .build()
        .unwrap();
    let report = StellarReport::from_profile(&profile).to_string();
    assert!(report.contains("Name: Sol"));
    assert!(report.contains("Mass: 1.000 M_sun"));
    assert!(report.contains("Frost Line"));
}

#[test]
fn stellar_module_integrates_with_engine() {
    let module = StellarModule::new(StellarModuleConfig::default());
    let mut engine = EngineBuilder::new()
        .with_seed(42)
        .register_module(Box::new(module))
        .build()
        .unwrap();

    engine.initialize().unwrap();
    engine.tick_fixed().unwrap();
    let diagnostics = engine.diagnostics();
    assert_eq!(
        diagnostics.registered_modules,
        vec!["worldsmith.stellar".to_string()]
    );
    assert_eq!(engine.state().stars.len(), 1);
    assert!(engine.latest_snapshot().is_some());
    assert_eq!(diagnostics.queued_event_count, 0);
}

#[test]
fn stellar_engine_runs_are_repeatable() {
    fn run() -> u64 {
        let mut engine = EngineBuilder::new()
            .with_seed(11)
            .register_module(Box::new(StellarModule::default()))
            .build()
            .unwrap();
        engine.initialize().unwrap();
        for _ in 0..3 {
            engine.tick_fixed().unwrap();
        }
        engine.state_fingerprint()
    }
    assert_eq!(run(), run());
}
