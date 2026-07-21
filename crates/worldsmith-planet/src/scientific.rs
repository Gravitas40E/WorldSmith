//! Scientific consistency checks for evolved planets.
//!
//! This module validates that derived planetary properties are physically
//! consistent and scientifically plausible. Every check has a physical
//! explanation — no arbitrary thresholds without justification.

use worldsmith_math::constants;
use worldsmith_models::Planet;
use worldsmith_traits::{ContractError, ContractResult};

/// Checks an evolved planet for scientific consistency.
///
/// Returns errors for physically impossible or contradictory states.
/// Warnings are not errors — borderline but plausible values pass.
pub fn check_planet_consistency(planet: &Planet) -> ContractResult<()> {
    let mass = planet.physical.mass_kg.value;
    let radius = planet.physical.radius_m.value;

    // Mass must be positive and finite
    if !mass.is_finite() || mass <= 0.0 {
        return Err(ContractError::InvalidInput(format!(
            "planet {} mass must be positive and finite: got {}",
            planet.id.0, mass
        )));
    }

    // Radius must be positive and finite
    if !radius.is_finite() || radius <= 0.0 {
        return Err(ContractError::InvalidInput(format!(
            "planet {} radius must be positive and finite: got {}",
            planet.id.0, radius
        )));
    }

    // Radius must not exceed mass by physically implausible margin
    // (density < 0.1 kg/m^3 is unphysical for any solid/gas body)
    let density = mass / (4.0 / 3.0 * std::f64::consts::PI * radius.powi(3));
    if density < 0.1 {
        return Err(ContractError::InvalidInput(format!(
            "planet {} density {:.3} kg/m^3 is unphysically low",
            planet.id.0, density
        )));
    }

    // Density must not exceed known dense matter (osmium ~ 22,600 kg/m^3)
    if density > 30_000.0 {
        return Err(ContractError::InvalidInput(format!(
            "planet {} density {:.3} kg/m^3 exceeds physically plausible maximum",
            planet.id.0, density
        )));
    }

    // Orbital distance must be positive
    let semi_major = planet.orbit.semi_major_axis_m.value;
    if !semi_major.is_finite() || semi_major <= 0.0 {
        return Err(ContractError::InvalidInput(format!(
            "planet {} semi-major axis must be positive and finite: got {}",
            planet.id.0, semi_major
        )));
    }

    // Surface gravity consistency
    if let Some(gravity) = &planet.physical.surface_gravity_m_s2 {
        let expected_g = constants::GRAVITATIONAL_CONSTANT * mass / radius.powi(2);
        let ratio = (gravity.value / expected_g).abs();
        if ratio > 1.5 && gravity.value > 0.0 {
            return Err(ContractError::InvalidInput(format!(
                "planet {} surface gravity {} m/s^2 deviates from G*M/R^2 = {} by factor {}",
                planet.id.0, gravity.value, expected_g, ratio
            )));
        }
    }

    // Atmosphere consistency
    if let Some(atmosphere) = &planet.atmosphere {
        if let Some(pressure) = &atmosphere.pressure_pa {
            if !pressure.value.is_finite() || pressure.value < 0.0 {
                return Err(ContractError::InvalidInput(format!(
                    "planet {} atmospheric pressure must be non-negative and finite: got {}",
                    planet.id.0, pressure.value
                )));
            }
            // Pressure cannot exceed arbitrary dense atmosphere threshold without
            // being a gas giant or having extreme mass
            if pressure.value > 1.0e9 {
                return Err(ContractError::InvalidInput(format!(
                    "planet {} atmospheric pressure {} Pa exceeds physically modeled range",
                    planet.id.0, pressure.value
                )));
            }
        }

        // Composition mole fractions should sum reasonably
        let composition_sum: f64 = atmosphere
            .composition
            .iter()
            .map(|g| g.abundance.value)
            .sum();
        if composition_sum > 1.5 && !atmosphere.composition.is_empty() {
            return Err(ContractError::InvalidInput(format!(
                "planet {} atmospheric composition fractions sum to {}, exceeding 1.0",
                planet.id.0, composition_sum
            )));
        }
    }

    // Climate consistency
    if let Some(climate) = &planet.climate {
        if let Some(temp) = &climate.average_temperature_k {
            if !temp.value.is_finite() || temp.value < 0.0 {
                return Err(ContractError::InvalidInput(format!(
                    "planet {} temperature must be non-negative and finite: got {}",
                    planet.id.0, temp.value
                )));
            }
            if temp.value > 10_000.0 {
                return Err(ContractError::InvalidInput(format!(
                    "planet {} temperature {} K exceeds plausible planetary range",
                    planet.id.0, temp.value
                )));
            }
        }

        // Ice coverage must be in [0, 1]
        if let Some(ice) = &climate.ice_coverage {
            if !ice.value.is_finite() || !(0.0..=1.0).contains(&ice.value) {
                return Err(ContractError::InvalidInput(format!(
                    "planet {} ice coverage {} must be in [0, 1]",
                    planet.id.0, ice.value
                )));
            }
        }

        // Humidity must be in [0, 1]
        if let Some(humidity) = &climate.humidity {
            if !humidity.value.is_finite() || !(0.0..=1.0).contains(&humidity.value) {
                return Err(ContractError::InvalidInput(format!(
                    "planet {} humidity {} must be in [0, 1]",
                    planet.id.0, humidity.value
                )));
            }
        }
    }

    // Magnetic field consistency
    if let Some(field) = &planet.magnetic_field {
        if let Some(strength) = &field.field_strength_t {
            if !strength.value.is_finite() || strength.value < 0.0 {
                return Err(ContractError::InvalidInput(format!(
                    "planet {} magnetic field strength must be non-negative and finite: got {}",
                    planet.id.0, strength.value
                )));
            }
            // ~10 Tesla is extreme for a planetary field (magnetars reach ~10^8 T,
            // but planet-scale dynamos max out around 0.01 T)
            if strength.value > 10.0 {
                return Err(ContractError::InvalidInput(format!(
                    "planet {} magnetic field {} T exceeds plausible planetary dynamo range",
                    planet.id.0, strength.value
                )));
            }
        }
    }

    // Ocean consistency with temperature
    if let Some(ocean) = &planet.ocean {
        if let Some(coverage) = &ocean.coverage {
            if !coverage.value.is_finite() || !(0.0..=1.0).contains(&coverage.value) {
                return Err(ContractError::InvalidInput(format!(
                    "planet {} ocean coverage {} must be in [0, 1]",
                    planet.id.0, coverage.value
                )));
            }
        }
        if let Some(climate) = &planet.climate {
            if let Some(temp) = &climate.average_temperature_k {
                // Water ocean with temperature far above boiling point
                if ocean.ocean_type == worldsmith_models::OceanType::Water && temp.value > 500.0 {
                    return Err(ContractError::InvalidInput(format!(
                        "planet {} has liquid water ocean at {} K, above water critical point",
                        planet.id.0, temp.value
                    )));
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use worldsmith_models::*;

    fn valid_planet() -> Planet {
        Planet {
            id: PlanetId(1),
            name: "Test".to_string(),
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
                density_kg_m3: None,
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
                rotation_period_s: None,
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
    fn valid_planet_passes_consistency() {
        let planet = valid_planet();
        assert!(check_planet_consistency(&planet).is_ok());
    }

    #[test]
    fn negative_mass_fails() {
        let mut planet = valid_planet();
        planet.physical.mass_kg.value = -1.0;
        assert!(check_planet_consistency(&planet).is_err());
    }

    #[test]
    fn nan_mass_fails() {
        let mut planet = valid_planet();
        planet.physical.mass_kg.value = f64::NAN;
        assert!(check_planet_consistency(&planet).is_err());
    }

    #[test]
    fn zero_radius_fails() {
        let mut planet = valid_planet();
        planet.physical.radius_m.value = 0.0;
        assert!(check_planet_consistency(&planet).is_err());
    }

    #[test]
    fn unphysical_density_fails() {
        let mut planet = valid_planet();
        // Very large radius with Earth mass gives implausibly low density
        planet.physical.radius_m.value = constants::EARTH_RADIUS * 1000.0;
        assert!(check_planet_consistency(&planet).is_err());
    }

    #[test]
    fn extreme_density_fails() {
        let mut planet = valid_planet();
        // Very small radius with Earth mass gives extreme density
        planet.physical.radius_m.value = constants::EARTH_RADIUS * 0.01;
        assert!(check_planet_consistency(&planet).is_err());
    }

    #[test]
    fn bad_gravity_deviates() {
        let mut planet = valid_planet();
        planet.physical.surface_gravity_m_s2 = Some(MeasuredValue {
            value: 100.0,
            unit: "m s^-2".to_string(),
            provenance: None,
        });
        assert!(check_planet_consistency(&planet).is_err());
    }

    #[test]
    fn bad_atmosphere_pressure_fails() {
        let mut planet = valid_planet();
        planet.atmosphere = Some(AtmosphericProperties {
            atmosphere_type: AtmosphereType::Standard,
            pressure_pa: Some(MeasuredValue {
                value: -100.0,
                unit: "Pa".to_string(),
                provenance: None,
            }),
            density_kg_m3: None,
            scale_height_m: None,
            layers: Vec::new(),
            composition: Vec::new(),
            cloud_coverage: None,
            greenhouse_gases: Vec::new(),
        });
        assert!(check_planet_consistency(&planet).is_err());
    }

    #[test]
    fn ocean_with_extreme_temperature_fails() {
        let mut planet = valid_planet();
        planet.ocean = Some(OceanProperties {
            ocean_type: OceanType::Water,
            coverage: Some(MeasuredValue {
                value: 0.5,
                unit: "fraction".to_string(),
                provenance: None,
            }),
            average_depth_m: None,
            composition: Vec::new(),
        });
        planet.climate = Some(ClimateProperties {
            climate_type: ClimateType::RunawayGreenhouse,
            average_temperature_k: Some(MeasuredValue {
                value: 600.0,
                unit: "K".to_string(),
                provenance: None,
            }),
            temperature_bands: Vec::new(),
            wind: None,
            humidity: None,
            ice_coverage: None,
            seasons: Vec::new(),
        });
        assert!(check_planet_consistency(&planet).is_err());
    }

    #[test]
    fn ice_out_of_range_fails() {
        let mut planet = valid_planet();
        planet.climate = Some(ClimateProperties {
            climate_type: ClimateType::Frozen,
            average_temperature_k: None,
            temperature_bands: Vec::new(),
            wind: None,
            humidity: None,
            ice_coverage: Some(MeasuredValue {
                value: 1.5,
                unit: "fraction".to_string(),
                provenance: None,
            }),
            seasons: Vec::new(),
        });
        assert!(check_planet_consistency(&planet).is_err());
    }

    #[test]
    fn humidity_out_of_range_fails() {
        let mut planet = valid_planet();
        planet.climate = Some(ClimateProperties {
            climate_type: ClimateType::Temperate,
            average_temperature_k: None,
            temperature_bands: Vec::new(),
            wind: None,
            humidity: Some(MeasuredValue {
                value: -0.1,
                unit: "fraction".to_string(),
                provenance: None,
            }),
            ice_coverage: None,
            seasons: Vec::new(),
        });
        assert!(check_planet_consistency(&planet).is_err());
    }

    #[test]
    fn extreme_magnetic_field_fails() {
        let mut planet = valid_planet();
        planet.magnetic_field = Some(MagneticFieldProperties {
            field_strength_t: Some(MeasuredValue {
                value: 100.0,
                unit: "T".to_string(),
                provenance: None,
            }),
            pole_orientation: None,
            magnetosphere_radius_m: None,
        });
        assert!(check_planet_consistency(&planet).is_err());
    }

    #[test]
    fn zero_semi_major_axis_fails() {
        let mut planet = valid_planet();
        planet.orbit.semi_major_axis_m.value = 0.0;
        assert!(check_planet_consistency(&planet).is_err());
    }

    #[test]
    fn check_consistency_function_works() {
        let planet = valid_planet();
        assert!(check_planet_consistency(&planet).is_ok());
    }
}
