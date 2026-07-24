//! Performance validation and measurement helpers.
//!
//! Record tick time, memory, and allocation counts for different planet
//! counts.  No optimization is performed; this is purely observational.

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

/// Performance measurement for a single scale test.
#[derive(Debug, Clone, PartialEq)]
pub struct PerformanceReport {
    /// Number of planets in the test.
    pub planet_count: usize,
    /// Total measured ticks.
    pub ticks: u64,
    /// Tick time in milliseconds.
    pub tick_time_ms: f64,
    /// Approximate memory usage in bytes (RSS snapshot).
    pub memory_bytes: u64,
    /// Approximate allocation count (from allocator if available).
    pub allocation_count: Option<u64>,
}

/// Run a performance validation at the specified planet count for 10 ticks.
pub fn measure_performance(planet_count: usize) -> PerformanceReport {
    let planets = build_planets(planet_count);
    let mut engine = EngineBuilder::new()
        .with_seed(7)
        .register_module(Box::new(StellarModule::default()))
        .register_module(Box::new(CoreEvolutionModule::default()))
        .register_module(Box::new(MantleEvolutionModule::default()))
        .register_module(Box::new(VolcanismModule::default()))
        .register_module(Box::new(PlateTectonicsModule::default()))
        .build()
        .expect("build performance engine");

    for (id, planet) in planets {
        engine.state_mut().planets.insert(id, planet);
    }
    engine.initialize().expect("initialize performance engine");

    let start = std::time::Instant::now();
    for _ in 0..10 {
        engine.tick_fixed().expect("tick performance engine");
    }
    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

    let memory_bytes = current_memory_bytes();
    let allocation_count = current_allocation_count();

    PerformanceReport {
        planet_count,
        ticks: 10,
        tick_time_ms: elapsed_ms / 10.0,
        memory_bytes,
        allocation_count,
    }
}

fn build_planets(count: usize) -> Vec<(PlanetId, Planet)> {
    (0..count)
        .map(|idx| {
            let id = PlanetId(idx as u64);
            let mass = 5.972e24 + (idx as f64) * 1e22;
            let radius = 6.371e6 + (idx as f64) * 1e5;
            let planet = Planet {
                id,
                name: format!("Perf-{idx}"),
                class: worldsmith_models::PlanetClass::Terrestrial,
                planet_type: PlanetType::Rocky,
                system_id: SystemId(1),
                physical: PhysicalProperties {
                    mass_kg: MeasuredValue {
                        value: mass,
                        unit: "kg".into(),
                        provenance: None,
                    },
                    radius_m: MeasuredValue {
                        value: radius,
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
            (id, planet)
        })
        .collect()
}

fn current_memory_bytes() -> u64 {
    #[cfg(target_os = "linux")]
    {
        let s = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
        for line in s.lines() {
            if let Some(rest) = line.strip_prefix("VmRSS:") {
                let kb = rest.trim().split_whitespace().next().unwrap_or("0");
                return kb.parse::<u64>().unwrap_or(0) * 1024;
            }
        }
        0
    }
    #[cfg(not(target_os = "linux"))]
    {
        0
    }
}

fn current_allocation_count() -> Option<u64> {
    None
}
