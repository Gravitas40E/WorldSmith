//! Integration tests for the evolution framework.
//!
//! These tests verify modules can be constructed, registered
//! with the engine, and executed in a deterministic order.

use worldsmith_engine::EngineBuilder;
use worldsmith_evolution::{
    AtmosphereModule, ClimateModule, CoreEvolutionModule, EvolutionPlugin, HydrologyModule,
    PlateTectonicsModule,
};
use worldsmith_math::Vector3;
use worldsmith_models::{
    AtmosphereState, BodyReference, HydrologyState, MeasuredValue, OrbitalProperties,
    PhysicalProperties, Planet, PlanetId, PlanetType, Star, StarId, SystemId,
};

fn earth_like_planet() -> Planet {
    Planet {
        id: PlanetId(1),
        name: "Integration".into(),
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

#[test]
fn core_runs_before_mantle_in_pipeline() {
    let engine = EvolutionPlugin::new()
        .register_with(EngineBuilder::new())
        .with_seed(7)
        .build()
        .unwrap();

    let pipeline = engine.diagnostics().active_pipeline;
    let core_idx = pipeline
        .iter()
        .position(|id| id == "worldsmith.evolution.core")
        .expect("core in pipeline");
    let mantle_idx = pipeline
        .iter()
        .position(|id| id == "worldsmith.evolution.mantle")
        .expect("mantle in pipeline");
    assert!(core_idx < mantle_idx, "core must run before mantle");
}

#[test]
fn mantle_runs_before_volcanism_in_pipeline() {
    let engine = EvolutionPlugin::new()
        .register_with(EngineBuilder::new())
        .with_seed(7)
        .build()
        .unwrap();

    let pipeline = engine.diagnostics().active_pipeline;
    let mantle_idx = pipeline
        .iter()
        .position(|id| id == "worldsmith.evolution.mantle")
        .expect("mantle in pipeline");
    let volcanism_idx = pipeline
        .iter()
        .position(|id| id == "worldsmith.evolution.volcanism")
        .expect("volcanism in pipeline");
    assert!(
        mantle_idx < volcanism_idx,
        "mantle must run before volcanism"
    );
}

#[test]
fn volcanism_runs_before_plate_tectonics_in_pipeline() {
    let engine = EvolutionPlugin::new()
        .register_with(EngineBuilder::new())
        .with_seed(7)
        .build()
        .unwrap();

    let pipeline = engine.diagnostics().active_pipeline;
    let volcanism_idx = pipeline
        .iter()
        .position(|id| id == "worldsmith.evolution.volcanism")
        .expect("volcanism in pipeline");
    let plate_idx = pipeline
        .iter()
        .position(|id| id == "worldsmith.evolution.plate_tectonics")
        .expect("plate tectonics in pipeline");
    assert!(
        volcanism_idx < plate_idx,
        "volcanism must run before plate tectonics"
    );
}

#[test]
fn plate_tectonics_runs_before_atmosphere_in_pipeline() {
    let engine = EvolutionPlugin::new()
        .register_with(EngineBuilder::new())
        .with_seed(7)
        .build()
        .unwrap();

    let pipeline = engine.diagnostics().active_pipeline;
    let plate_idx = pipeline
        .iter()
        .position(|id| id == "worldsmith.evolution.plate_tectonics")
        .expect("plate tectonics in pipeline");
    let atmosphere_idx = pipeline
        .iter()
        .position(|id| id == "worldsmith.evolution.atmosphere")
        .expect("atmosphere in pipeline");
    assert!(
        plate_idx < atmosphere_idx,
        "plate tectonics must run before atmosphere"
    );
}

#[test]
fn atmosphere_runs_before_hydrology_in_pipeline() {
    let engine = EvolutionPlugin::new()
        .register_with(EngineBuilder::new())
        .with_seed(7)
        .build()
        .unwrap();

    let pipeline = &engine.diagnostics().active_pipeline;
    let atmosphere_idx = pipeline
        .iter()
        .position(|id| id == "worldsmith.evolution.atmosphere")
        .expect("atmosphere in pipeline");
    let hydrology_idx = pipeline
        .iter()
        .position(|id| id == "worldsmith.evolution.hydrology")
        .expect("hydrology in pipeline");
    assert!(
        atmosphere_idx < hydrology_idx,
        "atmosphere must run before hydrology"
    );
}

#[test]
fn hydrology_module_executes_and_mutates_hydrology_state() {
    let mut engine = EngineBuilder::new()
        .with_seed(7)
        .register_module(Box::new(AtmosphereModule::default()))
        .register_module(Box::new(HydrologyModule::default()))
        .build()
        .unwrap();

    let planet_id = PlanetId(1);
    let star_id = StarId(1);
    let star = Star {
        id: star_id,
        name: "Sol".into(),
        spectral_type: worldsmith_models::SpectralType::G,
        class: worldsmith_models::StarClass::MainSequence,
        mass_kg: MeasuredValue {
            value: 1.989e30,
            unit: "kg".into(),
            provenance: None,
        },
        radius_m: MeasuredValue {
            value: 6.96e8,
            unit: "m".into(),
            provenance: None,
        },
        luminosity_w: MeasuredValue {
            value: 5.0e26,
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
            value: 0.0,
            unit: "dimensionless".into(),
            provenance: None,
        },
        rotation_period_s: None,
        age_s: None,
        position_m: worldsmith_math::Vector3::ZERO,
        velocity_m_s: worldsmith_math::Vector3::ZERO,
    };

    let planet = Planet {
        id: PlanetId(1),
        name: "Hydrology".into(),
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
        atmosphere_state: Some(AtmosphereState::default()),
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
    };

    engine.state_mut().planets.insert(planet_id, planet);
    engine.state_mut().stars.insert(star_id, star);
    let _ = engine.initialize();
    let _ = engine.tick(100.0);
    let _ = engine.tick(100.0);

    let updated = engine.state().planets.get(&planet_id).unwrap();
    assert!(updated.hydrology_state.is_some());
    let hydrology = updated.hydrology_state.as_ref().unwrap();
    assert!(hydrology.atmospheric_water_mass_kg > 0.0);
}

#[test]
fn hydrology_does_not_modify_atmosphere_state() {
    let mut engine = EngineBuilder::new()
        .with_seed(7)
        .register_module(Box::new(AtmosphereModule::default()))
        .register_module(Box::new(HydrologyModule::default()))
        .build()
        .unwrap();

    let planet_id = PlanetId(1);
    let star_id = StarId(1);
    let star = Star {
        id: star_id,
        name: "Sol".into(),
        spectral_type: worldsmith_models::SpectralType::G,
        class: worldsmith_models::StarClass::MainSequence,
        mass_kg: MeasuredValue {
            value: 1.989e30,
            unit: "kg".into(),
            provenance: None,
        },
        radius_m: MeasuredValue {
            value: 6.96e8,
            unit: "m".into(),
            provenance: None,
        },
        luminosity_w: MeasuredValue {
            value: 5.0e26,
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
            value: 0.0,
            unit: "dimensionless".into(),
            provenance: None,
        },
        rotation_period_s: None,
        age_s: None,
        position_m: worldsmith_math::Vector3::ZERO,
        velocity_m_s: worldsmith_math::Vector3::ZERO,
    };

    let planet = Planet {
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
            parent: BodyReference::Star(star_id),
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
        atmosphere_state: Some(AtmosphereState {
            atmospheric_mass_kg: 5.15e18,
            surface_pressure_pa: 101_325.0,
            mean_temperature_k: 288.0,
            atmosphere_composition: vec![],
        }),
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
    };

    engine.state_mut().planets.insert(planet_id, planet);
    engine.state_mut().stars.insert(star_id, star);
    let _ = engine.initialize();
    let _ = engine.tick(100.0);
    let _ = engine.tick(100.0);

    let updated = engine.state().planets.get(&planet_id).unwrap();
    assert!(updated.hydrology_state.is_some());
    assert!(
        updated
            .atmosphere_state
            .as_ref()
            .unwrap()
            .atmospheric_mass_kg
            == 5.15e18
    );
}

#[test]
fn snapshot_carries_plate_tectonics_state_after_tick() {
    let mut engine = EngineBuilder::new()
        .with_seed(7)
        .register_module(Box::new(CoreEvolutionModule::default()))
        .register_module(Box::new(PlateTectonicsModule::default()))
        .build()
        .unwrap();

    let planet_id = PlanetId(1);
    engine
        .state_mut()
        .planets
        .insert(planet_id, earth_like_planet());
    let _ = engine.initialize();
    let _ = engine.tick(100.0);

    let snapshot = engine.latest_snapshot().expect("snapshot");
    let planet = snapshot.planets.iter().find(|p| p.id == planet_id).unwrap();
    assert!(planet.planet.plate_tectonics.is_some());
}

#[test]
fn climate_module_executes_and_mutates_climate_state() {
    let mut engine = EngineBuilder::new()
        .with_seed(7)
        .register_module(Box::new(AtmosphereModule::default()))
        .register_module(Box::new(HydrologyModule::default()))
        .register_module(Box::new(ClimateModule::default()))
        .build()
        .unwrap();

    let planet_id = PlanetId(1);
    let star_id = StarId(1);
    let _system_id = SystemId(1);
    let star = Star {
        id: star_id,
        name: "Sol".into(),
        spectral_type: worldsmith_models::SpectralType::G,
        class: worldsmith_models::StarClass::MainSequence,
        mass_kg: MeasuredValue {
            value: 1.989e30,
            unit: "kg".into(),
            provenance: None,
        },
        radius_m: MeasuredValue {
            value: 6.96e8,
            unit: "m".into(),
            provenance: None,
        },
        luminosity_w: MeasuredValue {
            value: 5.0e26,
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
            value: 0.0,
            unit: "dimensionless".into(),
            provenance: None,
        },
        rotation_period_s: None,
        age_s: None,
        position_m: worldsmith_math::Vector3::ZERO,
        velocity_m_s: worldsmith_math::Vector3::ZERO,
    };

    let planet = Planet {
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
        geology: None,
        atmosphere: None,
        atmosphere_state: Some(AtmosphereState::default()),
        hydrology_state: Some(HydrologyState::default()),
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
    };

    let _ = engine.initialize();
    engine.state_mut().planets.insert(planet_id, planet);
    engine.state_mut().stars.insert(star_id, star);
    let _ = engine.tick(100.0);
    let _ = engine.tick(100.0);

    let updated = engine.state().planets.get(&planet_id).unwrap();
    assert!(updated.climate_state.is_some());
    let climate = updated.climate_state.as_ref().unwrap();
    assert!(climate.equilibrium_temperature_k > 0.0);
    assert!(climate.planetary_albedo <= 1.0);
}

#[test]
fn climate_runs_after_hydrology_in_pipeline() {
    let engine = EvolutionPlugin::new()
        .register_with(EngineBuilder::new())
        .with_seed(7)
        .build()
        .unwrap();

    let pipeline = engine.diagnostics().active_pipeline;
    let climate_idx = pipeline
        .iter()
        .position(|id| id == "worldsmith.evolution.climate")
        .expect("climate in pipeline");
    let hydrology_idx = pipeline
        .iter()
        .position(|id| id == "worldsmith.evolution.hydrology")
        .expect("hydrology in pipeline");

    assert!(
        climate_idx > hydrology_idx,
        "climate must run after hydrology"
    );
}

#[test]
fn climate_does_not_modify_atmosphere_state() {
    let mut engine = EngineBuilder::new()
        .with_seed(7)
        .register_module(Box::new(AtmosphereModule::default()))
        .register_module(Box::new(HydrologyModule::default()))
        .register_module(Box::new(ClimateModule::default()))
        .build()
        .unwrap();

    let planet_id = PlanetId(1);
    let star_id = StarId(1);
    let star = Star {
        id: star_id,
        name: "Sol".into(),
        spectral_type: worldsmith_models::SpectralType::G,
        class: worldsmith_models::StarClass::MainSequence,
        mass_kg: MeasuredValue {
            value: 1.989e30,
            unit: "kg".into(),
            provenance: None,
        },
        radius_m: MeasuredValue {
            value: 6.96e8,
            unit: "m".into(),
            provenance: None,
        },
        luminosity_w: MeasuredValue {
            value: 5.0e26,
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
            value: 0.0,
            unit: "dimensionless".into(),
            provenance: None,
        },
        rotation_period_s: None,
        age_s: None,
        position_m: worldsmith_math::Vector3::ZERO,
        velocity_m_s: worldsmith_math::Vector3::ZERO,
    };

    let planet = Planet {
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
            parent: BodyReference::Star(star_id),
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
        atmosphere_state: Some(AtmosphereState {
            atmospheric_mass_kg: 5.15e18,
            surface_pressure_pa: 101_325.0,
            mean_temperature_k: 288.0,
            atmosphere_composition: vec![],
        }),
        hydrology_state: Some(HydrologyState::default()),
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
    };

    engine.state_mut().planets.insert(planet_id, planet);
    engine.state_mut().stars.insert(star_id, star);
    let _ = engine.initialize();
    let _ = engine.tick(100.0);
    let _ = engine.tick(100.0);

    let updated = engine.state().planets.get(&planet_id).unwrap();
    assert!(updated.climate_state.is_some());
    assert!(
        updated
            .atmosphere_state
            .as_ref()
            .unwrap()
            .atmospheric_mass_kg
            == 5.15e18
    );
}

#[test]
fn snapshots_contain_climate_state() {
    let mut engine = EvolutionPlugin::new()
        .register_with(EngineBuilder::new())
        .with_seed(7)
        .build()
        .unwrap();

    let planet_id = PlanetId(1);
    let star_id = StarId(1);
    let star = Star {
        id: star_id,
        name: "Sol".into(),
        spectral_type: worldsmith_models::SpectralType::G,
        class: worldsmith_models::StarClass::MainSequence,
        mass_kg: MeasuredValue {
            value: 1.989e30,
            unit: "kg".into(),
            provenance: None,
        },
        radius_m: MeasuredValue {
            value: 6.96e8,
            unit: "m".into(),
            provenance: None,
        },
        luminosity_w: MeasuredValue {
            value: 5.0e26,
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
            value: 0.0,
            unit: "dimensionless".into(),
            provenance: None,
        },
        rotation_period_s: None,
        age_s: None,
        position_m: worldsmith_math::Vector3::ZERO,
        velocity_m_s: worldsmith_math::Vector3::ZERO,
    };

    let planet = Planet {
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
            parent: BodyReference::Star(star_id),
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
        atmosphere_state: Some(AtmosphereState::default()),
        hydrology_state: Some(HydrologyState::default()),
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
    };

    engine.state_mut().planets.insert(planet_id, planet);
    engine.state_mut().stars.insert(star_id, star);
    let _ = engine.initialize();
    let _ = engine.tick(100.0);
    let _ = engine.tick(100.0);

    let snapshot = engine.latest_snapshot().expect("snapshot");
    assert_ne!(snapshot.planets.len(), 0);
    let planet_snapshot = snapshot.planets.iter().find(|p| p.id == planet_id).unwrap();
    assert!(planet_snapshot.planet.climate_state.is_some());
}
