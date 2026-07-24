//! Scientific invariant tests.

use std::collections::BTreeMap;

use worldsmith_models::{Planet, PlanetId};
use worldsmith_validation::{validate_scientific_invariants, ScientificInvariantError};

#[test]
fn validate_invariants_passes_on_empty_map() {
    let planets = BTreeMap::new();
    assert!(validate_scientific_invariants(&planets).is_ok());
}

#[test]
fn validate_invariants_passes_with_valid_data() {
    let mut planets = BTreeMap::new();
    planets.insert(PlanetId(1), valid_planet());
    assert!(validate_scientific_invariants(&planets).is_ok());
}

#[test]
fn validate_invariants_detects_core_below_mantle() {
    let mut planets = BTreeMap::new();
    let mut planet = valid_planet();
    planet.interior = Some(worldsmith_models::InteriorState {
        age_seconds: 0.0,
        core_temperature: 100.0,
        mantle_temperature: 5000.0,
        heat_flux: 0.0,
        radiogenic_heat: 1.0,
        internal_heat: 1.0,
    });
    planets.insert(PlanetId(1), planet);
    let result = validate_scientific_invariants(&planets);
    assert!(result.is_err());
    match result.unwrap_err() {
        ScientificInvariantError::CoreBelowMantle { .. } => {}
        _ => panic!("expected CoreBelowMantle error"),
    }
}

#[test]
fn validate_invariants_detects_negative_volcanic_flux() {
    let mut planets = BTreeMap::new();
    let mut planet = valid_planet();
    planet.volcanism = Some(worldsmith_models::VolcanismState {
        volcanic_flux: -1.0,
        volcanic_activity: worldsmith_models::VolcanicActivity::None,
        magma_generation_rate: 0.0,
    });
    planets.insert(PlanetId(1), planet);
    let result = validate_scientific_invariants(&planets);
    assert!(result.is_err());
    match result.unwrap_err() {
        ScientificInvariantError::NegativeVolcanicFlux { .. } => {}
        _ => panic!("expected NegativeVolcanicFlux error"),
    }
}

#[test]
fn validate_invariants_detects_negative_plate_velocity() {
    let mut planets = BTreeMap::new();
    let mut planet = valid_planet();
    planet.plate_tectonics = Some(worldsmith_models::PlateTectonicsState {
        plate_velocity: -10.0,
        crustal_recycling_rate: 0.0,
        tectonic_activity: worldsmith_models::TectonicActivity::None,
    });
    planets.insert(PlanetId(1), planet);
    let result = validate_scientific_invariants(&planets);
    assert!(result.is_err());
    match result.unwrap_err() {
        ScientificInvariantError::NegativePlateVelocity { .. } => {}
        _ => panic!("expected NegativePlateVelocity error"),
    }
}

fn valid_planet() -> Planet {
    Planet {
        id: PlanetId(1),
        name: "Test".into(),
        class: worldsmith_models::PlanetClass::Terrestrial,
        planet_type: worldsmith_models::PlanetType::Rocky,
        system_id: worldsmith_models::SystemId(1),
        physical: worldsmith_models::PhysicalProperties {
            mass_kg: worldsmith_models::MeasuredValue {
                value: 1.0,
                unit: "kg".into(),
                provenance: None,
            },
            radius_m: worldsmith_models::MeasuredValue {
                value: 1.0,
                unit: "m".into(),
                provenance: None,
            },
            density_kg_m3: None,
            surface_gravity_m_s2: None,
        },
        orbit: worldsmith_models::OrbitalProperties {
            parent: worldsmith_models::BodyReference::Star(worldsmith_models::StarId(1)),
            semi_major_axis_m: worldsmith_models::MeasuredValue {
                value: 1.0,
                unit: "m".into(),
                provenance: None,
            },
            semi_minor_axis_m: None,
            eccentricity: worldsmith_models::MeasuredValue {
                value: 0.0,
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
        interior: Some(worldsmith_models::InteriorState {
            age_seconds: 0.0,
            core_temperature: 5000.0,
            mantle_temperature: 3000.0,
            heat_flux: 0.0,
            radiogenic_heat: 1.0,
            internal_heat: 1.0,
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
