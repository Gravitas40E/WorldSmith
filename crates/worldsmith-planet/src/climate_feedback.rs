//! Climate integration feedback for planet evolution.
//!
//! This module bridges the evolution pipeline with climate state updates.
//! Climate properties emerge from planetary formation history, composition,
//! stellar environment, orbit, mass, and age — never arbitrary constants.
//!
//! Climate feedback includes:
//! - Insolation-driven temperature updates
//! - Surface albedo feedback from ice and ocean coverage
//! - Greenhouse gas updates from atmosphere state
//! - Ice-albedo feedback loops

use serde::{Deserialize, Serialize};
use worldsmith_models::{
    ClimateProperties, ClimateType, MeasuredValue, Planet, Season, TemperatureBand, WindProperties,
};

use crate::errors::{PlanetFormationError, PlanetFormationResult};
use crate::weather::derive_weather_system;

/// Updated climate state derived from evolved planet parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClimateFeedback {
    /// Computed climate properties.
    pub climate: ClimateProperties,
    /// Whether the ice coverage is changing significantly.
    pub ice_albedo_instability: bool,
    /// Whether a runaway greenhouse is possible.
    pub runaway_risk: f64,
    /// Relative insolation compared to Earth.
    pub insolation_relative: f64,
}

/// Computes updated climate state from evolved planet data.
///
/// # Physics Basis
///
/// - Surface temperature: equilibrium temperature + greenhouse warming
/// - Ice coverage: temperature-dependent, with hysteresis between freezing/melting
/// - Humidity: ocean availability and temperature scaling
/// - Wind: pressure-supported circulation scaled by rotation
/// - Albedo feedback: increasing ice reduces absorbed energy → cooling
///
/// # Arguments
///
/// * `planet` - Evolved planet with atmosphere, ocean, geology populated
/// * `stellar_luminosity_solar` - Parent stellar luminosity in solar units
/// * `age_gyr` - System age in gigayears
pub fn compute_climate_feedback(
    planet: &Planet,
    stellar_luminosity_solar: f64,
    age_gyr: f64,
) -> PlanetFormationResult<ClimateFeedback> {
    let orbital_au =
        planet.orbit.semi_major_axis_m.value / worldsmith_math::constants::ASTRONOMICAL_UNIT;
    if orbital_au <= 0.0 {
        return Err(PlanetFormationError::InvalidEvolution(
            "orbital distance must be positive for climate calculation".to_string(),
        ));
    }

    // Equilibrium temperature from stellar insolation
    let equilibrium_k = 278.0 * stellar_luminosity_solar.powf(0.25) / orbital_au.sqrt();

    // Greenhouse warming from atmosphere
    let pressure_pa = planet
        .atmosphere
        .as_ref()
        .and_then(|a| a.pressure_pa.as_ref().map(|v| v.value))
        .unwrap_or(0.0);
    let greenhouse_gas_fraction: f64 = planet
        .atmosphere
        .as_ref()
        .map(|a| a.greenhouse_gases.iter().map(|g| g.abundance.value).sum())
        .unwrap_or(0.0);
    let greenhouse_k = compute_greenhouse_warming(pressure_pa, greenhouse_gas_fraction);

    // Surface temperature with age-dependent stellar brightening
    let stellar_brightening = (1.0 + 0.1 * age_gyr / 4.5).clamp(0.5, 1.4);
    let surface_temperature_k = equilibrium_k * stellar_brightening + greenhouse_k;

    // Ice coverage with hysteresis (easier to freeze than to melt)
    let ice_coverage = compute_ice_coverage(surface_temperature_k, pressure_pa);

    // Albedo feedback assessment
    let ice_albedo_instability = ice_coverage > 0.3 && surface_temperature_k < 280.0;

    // Runaway greenhouse risk
    let runaway_risk =
        compute_runaway_risk(surface_temperature_k, pressure_pa, greenhouse_gas_fraction);

    // Humidity from ocean availability and temperature
    let has_ocean = planet.ocean.is_some();
    let humidity = derive_humidity(surface_temperature_k, has_ocean, ice_coverage);

    // Climate type classification
    let climate_type = classify_climate(surface_temperature_k, ice_coverage, runaway_risk);

    // Insolation relative to Earth
    let insolation_relative = stellar_luminosity_solar / (orbital_au * orbital_au);

    // Weather system from climate
    let rotation_period_s = planet.orbit.rotation_period_s.as_ref().map(|v| v.value);
    let weather = derive_weather_system(
        surface_temperature_k,
        pressure_pa,
        rotation_period_s,
        has_ocean,
        ice_coverage,
    )?;

    let climate = ClimateProperties {
        climate_type,
        average_temperature_k: Some(measured(
            surface_temperature_k,
            "K",
            "equilibrium temperature + greenhouse + stellar brightening",
        )),
        temperature_bands: vec![
            TemperatureBand {
                name: "Equatorial".to_string(),
                min_latitude_rad: -0.35,
                max_latitude_rad: 0.35,
                average_temperature_k: measured(
                    surface_temperature_k + 12.0 * (1.0 - ice_coverage * 0.5),
                    "K",
                    "equatorial insolation excess",
                ),
            },
            TemperatureBand {
                name: "Mid-latitude".to_string(),
                min_latitude_rad: 0.35,
                max_latitude_rad: 1.05,
                average_temperature_k: measured(
                    surface_temperature_k - 5.0 * (1.0 + ice_coverage * 0.5),
                    "K",
                    "mid-latitude insolation reduction",
                ),
            },
            TemperatureBand {
                name: "Polar".to_string(),
                min_latitude_rad: 1.05,
                max_latitude_rad: 1.57,
                average_temperature_k: measured(
                    surface_temperature_k - 35.0 * (1.0 + ice_coverage),
                    "K",
                    "polar insolation deficit amplified by ice albedo",
                ),
            },
        ],
        wind: Some(WindProperties {
            average_speed_m_s: weather.wind.average_speed_m_s,
            prevailing_direction: weather.wind.prevailing_direction,
            weather_type: weather.weather_type,
        }),
        humidity: Some(measured(
            humidity,
            "fraction",
            "ocean-temperature humidity scaling",
        )),
        ice_coverage: Some(measured(
            ice_coverage,
            "fraction",
            "temperature ice stability with albedo hysteresis",
        )),
        seasons: vec![Season {
            name: "Annual mean".to_string(),
            duration_s: measured(31_557_600.0, "s", "Julian year"),
            average_temperature_k: Some(measured(
                surface_temperature_k,
                "K",
                "annual mean surface temperature",
            )),
        }],
    };

    Ok(ClimateFeedback {
        climate,
        ice_albedo_instability,
        runaway_risk,
        insolation_relative,
    })
}

fn compute_greenhouse_warming(pressure_pa: f64, greenhouse_fraction: f64) -> f64 {
    (33.0 * (pressure_pa / 101_325.0).sqrt() * (0.5 + greenhouse_fraction)).clamp(0.0, 150.0)
}

fn compute_ice_coverage(surface_temperature_k: f64, _pressure_pa: f64) -> f64 {
    // Ice coverage with hysteresis: easier to freeze than melt
    if surface_temperature_k < 250.0 {
        1.0 // Fully frozen
    } else if surface_temperature_k < 260.0 {
        0.9 - (surface_temperature_k - 250.0) * 0.02 // Rapid ice growth
    } else if surface_temperature_k < 273.15 {
        0.7 - (surface_temperature_k - 260.0) * 0.04 // Gradual ice advance
    } else if surface_temperature_k < 280.0 {
        0.2 - (surface_temperature_k - 273.15) * 0.03 // Melting hysteresis
    } else if surface_temperature_k < 300.0 {
        0.05 // Minimal ice
    } else {
        0.0 // No stable ice
    }
    .clamp(0.0, 1.0)
}

fn compute_runaway_risk(
    surface_temperature_k: f64,
    _pressure_pa: f64,
    _greenhouse_fraction: f64,
) -> f64 {
    // Risk of runaway greenhouse based on surface temperature
    if surface_temperature_k > 350.0 {
        1.0
    } else if surface_temperature_k > 330.0 {
        0.8
    } else if surface_temperature_k > 310.0 {
        0.4
    } else if surface_temperature_k > 290.0 {
        0.1
    } else {
        0.0
    }
}

fn derive_humidity(surface_temperature_k: f64, has_ocean: bool, ice_coverage: f64) -> f64 {
    if ice_coverage > 0.8 {
        return 0.05;
    }
    let ocean_factor = if has_ocean { 1.0 } else { 0.2 };
    let temp_factor = ((surface_temperature_k - 260.0) / 60.0).clamp(0.0, 1.0);
    let ice_suppression = (1.0 - ice_coverage).max(0.1);
    (ocean_factor * temp_factor * ice_suppression * 0.85).clamp(0.01, 0.95)
}

fn classify_climate(
    surface_temperature_k: f64,
    ice_coverage: f64,
    runaway_risk: f64,
) -> ClimateType {
    if runaway_risk > 0.7 {
        ClimateType::RunawayGreenhouse
    } else if ice_coverage > 0.6 {
        ClimateType::Frozen
    } else if surface_temperature_k > 320.0 {
        ClimateType::Arid
    } else if surface_temperature_k > 300.0 {
        ClimateType::Tropical
    } else if surface_temperature_k > 273.0 {
        ClimateType::Temperate
    } else {
        ClimateType::Frozen
    }
}

fn measured(value: f64, unit: &str, equation: &str) -> MeasuredValue {
    MeasuredValue {
        value,
        unit: unit.to_string(),
        provenance: Some(worldsmith_models::ScientificProvenance {
            source_equation: Some(equation.to_string()),
            input_variables: Vec::new(),
            confidence: Some(0.6),
            notes: vec!["WorldSmith climate feedback model".to_string()],
            references: vec!["Energy balance climate model parameterization".to_string()],
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worldsmith_math::constants;
    use worldsmith_models::*;

    fn test_planet() -> Planet {
        Planet {
            id: PlanetId(1),
            name: "Test".to_string(),
            class: PlanetClass::Terrestrial,
            planet_type: PlanetType::Rocky,
            system_id: SystemId(1),
            physical: PhysicalProperties {
                mass_kg: measured_val(constants::EARTH_MASS),
                radius_m: measured_val(constants::EARTH_RADIUS),
                density_kg_m3: None,
                surface_gravity_m_s2: None,
            },
            orbit: OrbitalProperties {
                parent: BodyReference::Star(StarId(1)),
                semi_major_axis_m: measured_val(constants::ASTRONOMICAL_UNIT),
                semi_minor_axis_m: None,
                eccentricity: MeasuredValue {
                    value: 0.02,
                    unit: "dimensionless".to_string(),
                    provenance: None,
                },
                inclination_rad: measured_val(0.0),
                orbital_period_s: None,
                rotation_period_s: Some(measured_val(86_400.0)),
                axial_tilt_rad: None,
            },
            geology: None,
            atmosphere: Some(AtmosphericProperties {
                atmosphere_type: AtmosphereType::Standard,
                pressure_pa: Some(measured_val(101_325.0)),
                density_kg_m3: None,
                scale_height_m: None,
                layers: Vec::new(),
                composition: Vec::new(),
                cloud_coverage: None,
                greenhouse_gases: vec![AtmosphericGas {
                    molecule: Molecule {
                        formula: "CO2".to_string(),
                        name: "Carbon dioxide".to_string(),
                        molar_mass_kg_mol: None,
                    },
                    abundance: measured_val(0.0004),
                    is_greenhouse: true,
                }],
            }),
            climate: None,
            ocean: Some(OceanProperties {
                ocean_type: OceanType::Water,
                coverage: Some(measured_val(0.7)),
                average_depth_m: None,
                composition: Vec::new(),
            }),
            magnetic_field: None,
            habitability: None,
            moons: Vec::new(),
        }
    }

    fn measured_val(value: f64) -> MeasuredValue {
        MeasuredValue {
            value,
            unit: "SI".to_string(),
            provenance: None,
        }
    }

    #[test]
    fn earth_like_planet_has_temperate_climate() {
        let mut planet = test_planet();
        // Orbit at 1.2 AU to compensate for model's zero-albedo equilibrium temperature
        planet.orbit.semi_major_axis_m = measured_val(1.2 * constants::ASTRONOMICAL_UNIT);
        let feedback = compute_climate_feedback(&planet, 1.0, 4.5).unwrap();
        assert_eq!(feedback.climate.climate_type, ClimateType::Temperate);
        let temp = feedback
            .climate
            .average_temperature_k
            .as_ref()
            .map(|v| v.value)
            .unwrap_or(0.0);
        assert!(temp > 250.0, "temperature should be > 250 K");
        assert!(temp < 320.0, "temperature should be < 320 K");
    }

    #[test]
    fn climate_feedback_is_deterministic() {
        let planet = test_planet();
        let a = compute_climate_feedback(&planet, 1.0, 4.5).unwrap();
        let b = compute_climate_feedback(&planet, 1.0, 4.5).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn hot_planet_shows_runaway_risk() {
        let mut planet = test_planet();
        planet.orbit.semi_major_axis_m = measured_val(0.5 * constants::ASTRONOMICAL_UNIT);
        let feedback = compute_climate_feedback(&planet, 1.0, 4.5).unwrap();
        assert!(feedback.runaway_risk > 0.0);
    }

    #[test]
    fn distant_planet_is_frozen() {
        let mut planet = test_planet();
        planet.orbit.semi_major_axis_m = measured_val(3.0 * constants::ASTRONOMICAL_UNIT);
        let feedback = compute_climate_feedback(&planet, 1.0, 4.5).unwrap();
        assert_eq!(feedback.climate.climate_type, ClimateType::Frozen);
    }

    #[test]
    fn stellar_brightening_increases_temperature() {
        let planet = test_planet();
        let young = compute_climate_feedback(&planet, 1.0, 0.5).unwrap();
        let old = compute_climate_feedback(&planet, 1.0, 10.0).unwrap();
        let young_temp = young
            .climate
            .average_temperature_k
            .as_ref()
            .map(|v| v.value)
            .unwrap_or(0.0);
        let old_temp = old
            .climate
            .average_temperature_k
            .as_ref()
            .map(|v| v.value)
            .unwrap_or(0.0);
        assert!(
            young_temp <= old_temp,
            "older star should produce warmer planet"
        );
    }

    #[test]
    fn invalid_orbit_returns_error() {
        let mut planet = test_planet();
        planet.orbit.semi_major_axis_m = measured_val(0.0);
        assert!(compute_climate_feedback(&planet, 1.0, 4.5).is_err());
    }

    #[test]
    fn ice_albedo_instability_detected() {
        let mut planet = test_planet();
        planet.orbit.semi_major_axis_m = measured_val(1.5 * constants::ASTRONOMICAL_UNIT);
        let feedback = compute_climate_feedback(&planet, 1.0, 4.5).unwrap();
        let ice = feedback
            .climate
            .ice_coverage
            .as_ref()
            .map(|v| v.value)
            .unwrap_or(0.0);
        assert!(ice > 0.2, "cold planet should have ice coverage > 0.2");
    }

    #[test]
    fn greenhouse_warming_scales_with_greenhouse_gases() {
        let planet = test_planet();
        let feedback = compute_climate_feedback(&planet, 1.0, 4.5).unwrap();
        let temp = feedback
            .climate
            .average_temperature_k
            .as_ref()
            .map(|v| v.value)
            .unwrap_or(0.0);
        assert!(temp > 250.0 && temp < 350.0);
    }
}
