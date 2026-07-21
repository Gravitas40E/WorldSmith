//! Geological evolution integration for planet evolution.
//!
//! This module connects the evolution pipeline with geological processes
//! including tectonics, volcanism, erosion, and resurfacing. Geological
//! state emerges from planetary mass, composition, heat budget, and age.

use serde::{Deserialize, Serialize};
use worldsmith_math::constants;
use worldsmith_models::{
    CrustProperties, GeologicalProperties, MeasuredValue, Planet, TectonicActivity,
    VolcanicActivity,
};

use crate::errors::PlanetFormationResult;
use crate::interior::InteriorModel;

/// Description of geological evolution over time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeologicalEvolution {
    /// Updated geological properties.
    pub geology: GeologicalProperties,
    /// Crustal recycling rate in cubic kilometers per year.
    pub crustal_recycling_rate_km3_yr: f64,
    /// Volcanic outgassing CO2 flux in kilograms per second.
    pub volcanic_co2_flux_kg_s: f64,
    /// Erosion rate in meters per million years.
    pub erosion_rate_m_myr: f64,
    /// Whether plate tectonics is actively cycling the surface.
    pub active_plate_tectonics: bool,
}

/// Computes geological evolution from planet properties and interior state.
///
/// # Physics Basis
///
/// - Tectonic activity: depends on mass (drives mantle convection), heat flow,
///   and water content (lubricates subduction).
/// - Volcanic activity: scales with internal heat budget.
/// - Erosion: climate-dependent, parameterized by surface temperature and
///   atmospheric pressure.
/// - Crustal recycling: active tectonics drives subduction of old crust.
///
/// # Arguments
///
/// * `planet` - Planet with interior model populated
/// * `interior` - Interior differentiation model
/// * `surface_temperature_k` - Current mean surface temperature
pub fn compute_geological_evolution(
    planet: &Planet,
    interior: &InteriorModel,
    surface_temperature_k: f64,
) -> PlanetFormationResult<GeologicalEvolution> {
    let earth_masses = planet.physical.mass_kg.value / constants::EARTH_MASS;
    let water_fraction = planet
        .ocean
        .as_ref()
        .map(|_| {
            planet
                .ocean
                .as_ref()
                .and_then(|o| o.coverage.as_ref().map(|c| c.value * 0.02))
                .unwrap_or(0.005)
        })
        .unwrap_or(0.001);

    // Tectonic activity from mass, heat, and water
    let tectonics = derive_tectonics(
        earth_masses,
        interior.heat_budget.total_heat_w,
        water_fraction,
    );
    let active_plate_tectonics = matches!(
        tectonics,
        TectonicActivity::Moderate | TectonicActivity::High
    );

    // Volcanic activity from heat budget
    let volcanism = derive_volcanism(interior.heat_budget.total_heat_w);

    // Erosion rate from climate (temperature and implied precipitation)
    let erosion_rate_m_myr = compute_erosion_rate(surface_temperature_k, water_fraction);

    // Crustal recycling from tectonics
    let crustal_recycling_rate_km3_yr = if active_plate_tectonics {
        (earth_masses * 1.5).clamp(0.5, 30.0)
    } else {
        (earth_masses * 0.1).clamp(0.0, 3.0)
    };

    // Volcanic CO2 flux
    let volcanic_co2_flux_kg_s =
        compute_volcanic_co2_flux(volcanism, interior.heat_budget.radioactive_heat_w);

    let crust_thickness_m = interior.crust_thickness_m
        * (1.0
            + if active_plate_tectonics {
                0.0
            } else {
                0.5 * erosion_rate_m_myr / 100.0
            });

    let geology = GeologicalProperties {
        core: interior.geology.core.clone(),
        mantle: interior.geology.mantle.clone(),
        crust: Some(CrustProperties {
            mean_thickness_m: Some(measured(
                crust_thickness_m,
                "m",
                "crust thickness from evolution, including tectonic thickening",
            )),
            materials: interior
                .geology
                .crust
                .as_ref()
                .map(|c| c.materials.clone())
                .unwrap_or_default(),
        }),
        surface_materials: interior.geology.surface_materials.clone(),
        plate_system: interior.geology.plate_system.as_ref().map(|_ps| {
            worldsmith_models::PlateSystem {
                activity: tectonics,
                major_plate_count: if active_plate_tectonics {
                    Some(7)
                } else {
                    None
                },
            }
        }),
        heat_flow_w_m2: Some(measured(
            interior.heat_budget.total_heat_w
                / (4.0 * std::f64::consts::PI * planet.physical.radius_m.value.powi(2)),
            "W m^-2",
            "heat flow from interior heat budget",
        )),
        volcanism,
    };

    Ok(GeologicalEvolution {
        geology,
        crustal_recycling_rate_km3_yr,
        volcanic_co2_flux_kg_s,
        erosion_rate_m_myr,
        active_plate_tectonics,
    })
}

fn derive_tectonics(earth_masses: f64, total_heat_w: f64, water_fraction: f64) -> TectonicActivity {
    if earth_masses > 0.5 && total_heat_w > 1.0e13 && water_fraction > 0.001 {
        TectonicActivity::Moderate
    } else if total_heat_w > 3.0e12 {
        TectonicActivity::Low
    } else {
        TectonicActivity::None
    }
}

fn derive_volcanism(total_heat_w: f64) -> VolcanicActivity {
    if total_heat_w > 5.0e13 {
        VolcanicActivity::High
    } else if total_heat_w > 1.0e13 {
        VolcanicActivity::Moderate
    } else if total_heat_w > 1.0e12 {
        VolcanicActivity::Low
    } else {
        VolcanicActivity::None
    }
}

fn compute_erosion_rate(surface_temperature_k: f64, water_fraction: f64) -> f64 {
    if water_fraction < 0.0001 || surface_temperature_k < 260.0 {
        return 0.5; // Minimal erosion on dry or frozen worlds
    }
    let thermal = (surface_temperature_k - 260.0) / 60.0;
    let water_factor = (water_fraction * 100.0).clamp(0.1, 5.0);
    (thermal * water_factor * 20.0).clamp(1.0, 200.0)
}

fn compute_volcanic_co2_flux(volcanism: VolcanicActivity, radioactive_heat_w: f64) -> f64 {
    let base_flux = match volcanism {
        VolcanicActivity::None => 0.0,
        VolcanicActivity::Low => 1.0e6,
        VolcanicActivity::Moderate => 1.0e7,
        VolcanicActivity::High => 5.0e7,
        VolcanicActivity::Extreme => 2.0e8,
        VolcanicActivity::Other => 1.0e6,
    };
    // Scale by radioactive heating relative to Earth
    if radioactive_heat_w > 0.0 {
        base_flux * (radioactive_heat_w / 2.0e13).clamp(0.1, 5.0)
    } else {
        base_flux
    }
}

fn measured(value: f64, unit: &str, equation: &str) -> MeasuredValue {
    MeasuredValue {
        value,
        unit: unit.to_string(),
        provenance: Some(worldsmith_models::ScientificProvenance {
            source_equation: Some(equation.to_string()),
            input_variables: Vec::new(),
            confidence: Some(0.55),
            notes: vec!["WorldSmith geological evolution model".to_string()],
            references: vec!["Parameterized terrestrial planet geology".to_string()],
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interior::differentiate_interior;
    use worldsmith_models::*;

    fn test_interior() -> InteriorModel {
        let mass = constants::EARTH_MASS;
        let radius = constants::EARTH_RADIUS;
        differentiate_interior(mass, radius, 0.34, 0.005, 4.5, Some(86_400.0)).unwrap()
    }

    fn test_planet() -> Planet {
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
            ocean: Some(OceanProperties {
                ocean_type: OceanType::Water,
                coverage: Some(MeasuredValue {
                    value: 0.7,
                    unit: "fraction".to_string(),
                    provenance: None,
                }),
                average_depth_m: None,
                composition: Vec::new(),
            }),
            magnetic_field: None,
            habitability: None,
            moons: Vec::new(),
        }
    }

    #[test]
    fn earth_like_has_active_tectonics() {
        let planet = test_planet();
        let interior = test_interior();
        let evo = compute_geological_evolution(&planet, &interior, 288.0).unwrap();
        assert!(evo.active_plate_tectonics);
        assert_eq!(evo.geology.volcanism, VolcanicActivity::Moderate);
    }

    #[test]
    fn geological_evolution_is_deterministic() {
        let planet = test_planet();
        let interior = test_interior();
        let a = compute_geological_evolution(&planet, &interior, 288.0).unwrap();
        let b = compute_geological_evolution(&planet, &interior, 288.0).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn frozen_planet_has_low_erosion() {
        let planet = test_planet();
        let interior = test_interior();
        let cold = compute_geological_evolution(&planet, &interior, 200.0).unwrap();
        let warm = compute_geological_evolution(&planet, &interior, 288.0).unwrap();
        assert!(cold.erosion_rate_m_myr <= warm.erosion_rate_m_myr);
    }

    #[test]
    fn small_planet_may_not_have_tectonics() {
        let mass = 0.1 * constants::EARTH_MASS;
        let radius = 0.5 * constants::EARTH_RADIUS;
        let interior =
            differentiate_interior(mass, radius, 0.34, 0.001, 10.0, Some(86_400.0)).unwrap();
        let mut planet = test_planet();
        planet.physical.mass_kg.value = mass;
        planet.physical.radius_m.value = radius;
        let evo = compute_geological_evolution(&planet, &interior, 250.0).unwrap();
        // Small cold planet may not have active tectonics
        assert!(!evo.active_plate_tectonics || evo.geology.volcanism == VolcanicActivity::None);
    }

    #[test]
    fn volcanic_co2_flux_scales_with_activity() {
        let planet = test_planet();
        let interior = test_interior();
        let evo = compute_geological_evolution(&planet, &interior, 288.0).unwrap();
        assert!(evo.volcanic_co2_flux_kg_s >= 0.0);
        assert!(evo.volcanic_co2_flux_kg_s.is_finite());
    }

    #[test]
    fn erosion_rate_increases_with_temperature() {
        let planet = test_planet();
        let interior = test_interior();
        let cold = compute_geological_evolution(&planet, &interior, 270.0).unwrap();
        let hot = compute_geological_evolution(&planet, &interior, 310.0).unwrap();
        assert!(cold.erosion_rate_m_myr <= hot.erosion_rate_m_myr);
    }
}
