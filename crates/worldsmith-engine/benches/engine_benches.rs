use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use worldsmith_engine::EngineBuilder;
use worldsmith_models::{Planet, PlanetId, PlanetType};

fn bench_planet_generation(c: &mut Criterion) {
    c.bench_function("planet_generation", |b| {
        b.iter(|| black_box(simple_planet()))
    });
}

fn bench_ticks(c: &mut Criterion) {
    let mut group = c.benchmark_group("ticks");
    for ticks in [100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(ticks), ticks, |b, &ticks| {
            b.iter(|| {
                let mut engine = EngineBuilder::new().with_seed(12345).build().unwrap();
                engine
                    .state_mut()
                    .planets
                    .insert(PlanetId(1), simple_planet());
                engine.initialize().unwrap();
                for _ in 0..ticks {
                    engine.tick(100.0).unwrap();
                }
                engine.latest_snapshot().unwrap().clone()
            });
        });
    }
    group.finish();
}

fn bench_snapshot(c: &mut Criterion) {
    let mut engine = EngineBuilder::new().with_seed(12345).build().unwrap();
    engine
        .state_mut()
        .planets
        .insert(PlanetId(1), simple_planet());
    engine.initialize().unwrap();
    engine.tick(100.0).unwrap();

    c.bench_function("snapshot_creation", |b| {
        b.iter(|| engine.latest_snapshot().unwrap().clone())
    });
}

fn simple_planet() -> Planet {
    Planet {
        id: PlanetId(1),
        name: "Bench World".into(),
        class: worldsmith_models::PlanetClass::Terrestrial,
        planet_type: PlanetType::Rocky,
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

criterion_group!(
    benches,
    bench_planet_generation,
    bench_ticks,
    bench_snapshot
);
criterion_main!(benches);
