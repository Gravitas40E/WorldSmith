//! Planetary evolution pipeline after formation.
//!
//! This module evolves formed planets into geophysical worlds by computing
//! interior differentiation, atmosphere, climate, ocean, weather, geological
//! feedback, magnetic field, and habitability. Every property emerges from
//! formation history, composition, stellar environment, orbit, mass, and age.

use serde::{Deserialize, Serialize};
use worldsmith_math::constants;
use worldsmith_models::{
    AtmosphereType, AtmosphericGas, AtmosphericLayer, AtmosphericProperties, ClimateType,
    MeasuredValue, Molecule, Planet, WeatherType,
};

use crate::{
    climate_feedback::compute_climate_feedback,
    errors::PlanetFormationResult,
    evolution_validation::{validate_evolution_inputs, validate_planet_for_evolution},
    geological_evolution::compute_geological_evolution,
    habitability::assess_habitability,
    hydrology::derive_ocean_properties,
    interior::{differentiate_interior, HeatBudget, InteriorModel},
    weather::derive_weather_system,
};

/// Evolved planet plus scientific timeline and derived systems.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanetEvolutionOutput {
    /// Planet with geology, atmosphere, climate, ocean, magnetic field, and habitability populated.
    pub planet: Planet,
    /// Interior model used to populate the planet.
    pub interior: InteriorModel,
    /// Timeline entries in chronological order.
    pub timeline: Vec<EvolutionTimelineEntry>,
}

/// Chronological explanation of planetary evolution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvolutionTimelineEntry {
    /// Time after formation in megayears.
    pub time_myr: f64,
    /// Event title.
    pub event: String,
    /// Scientific explanation.
    pub explanation: String,
}

/// Evolves a formed planet into an initial geophysical world state.
///
/// # Physics Basis
///
/// The evolution pipeline derives all planetary properties from:
///
/// - **Formation history**: mass, radius, density, composition fractions
/// - **Orbital context**: semi-major axis, rotation period
/// - **Stellar environment**: luminosity drives insolation and equilibrium temperature
/// - **Planetary age**: radioactive decay, cooling, and stellar brightening
///
/// Pipeline order:
/// 1. Validate inputs
/// 2. Interior differentiation (core, mantle, crust, heat, magnetic field)
/// 3. Atmosphere generation (outgassing, retention, composition)
/// 4. Climate feedback (equilibrium temperature, greenhouse, ice, humidity)
/// 5. Weather system (winds, precipitation, storms)
/// 6. Hydrology (ocean condensation)
/// 7. Geological evolution (tectonics, volcanism, erosion)
/// 8. Habitability assessment
/// 9. Timeline construction
pub fn evolve_planet(
    mut planet: Planet,
    stellar_luminosity_solar: f64,
    age_gyr: f64,
) -> PlanetFormationResult<PlanetEvolutionOutput> {
    let mass = planet.physical.mass_kg.value;
    let radius = planet.physical.radius_m.value;
    let orbital_au = planet.orbit.semi_major_axis_m.value / constants::ASTRONOMICAL_UNIT;

    // Step 1: Validate inputs
    validate_evolution_inputs(stellar_luminosity_solar, age_gyr)?;
    validate_planet_for_evolution(mass, radius, orbital_au)?;

    // Step 2: Interior differentiation
    let density = planet
        .physical
        .density_kg_m3
        .as_ref()
        .map(|v| v.value)
        .unwrap_or(5_000.0);
    let metal_fraction = if density > 5_000.0 { 0.34 } else { 0.22 };
    let water_fraction = match planet.planet_type {
        worldsmith_models::PlanetType::Ocean => 0.08,
        worldsmith_models::PlanetType::Ice => 0.20,
        _ => 0.005,
    };
    let interior = differentiate_interior(
        mass,
        radius,
        metal_fraction,
        water_fraction,
        age_gyr,
        planet.orbit.rotation_period_s.as_ref().map(|v| v.value),
    )?;
    planet.geology = Some(interior.geology.clone());
    planet.magnetic_field = Some(interior.magnetic_field.clone());

    // Step 3: Atmosphere generation
    let atmosphere = derive_atmosphere(&planet, &interior.heat_budget, water_fraction);
    let pressure = atmosphere
        .pressure_pa
        .as_ref()
        .map(|v| v.value)
        .unwrap_or(0.0);
    planet.atmosphere = Some(atmosphere);

    // Step 4: Climate feedback
    let climate_feedback = compute_climate_feedback(&planet, stellar_luminosity_solar, age_gyr)?;
    let surface_temperature_k = climate_feedback
        .climate
        .average_temperature_k
        .as_ref()
        .map(|v| v.value)
        .unwrap_or(278.0);
    let runaway_risk = climate_feedback.runaway_risk;
    let ice_albedo_instability = climate_feedback.ice_albedo_instability;
    planet.climate = Some(climate_feedback.climate);

    // Step 5: Weather system (merged into climate)
    if let Some(ref mut climate) = planet.climate {
        let has_ocean = planet.ocean.is_some();
        let rotation_period_s = planet.orbit.rotation_period_s.as_ref().map(|v| v.value);
        let weather = derive_weather_system(
            surface_temperature_k,
            pressure,
            rotation_period_s,
            has_ocean,
            climate
                .ice_coverage
                .as_ref()
                .map(|v| v.value)
                .unwrap_or(0.0),
        )?;
        climate.wind = Some(weather.wind);
        if weather.weather_type != WeatherType::Calm {
            climate.wind.as_mut().unwrap().weather_type = weather.weather_type;
        }
    }

    // Step 6: Hydrology
    planet.ocean = derive_ocean_properties(surface_temperature_k, pressure, water_fraction);

    // Step 7: Geological evolution (post-climate feedback)
    let geological_evolution =
        compute_geological_evolution(&planet, &interior, surface_temperature_k)?;
    let active_plate_tectonics = geological_evolution.active_plate_tectonics;
    planet.geology = Some(geological_evolution.geology);

    // Step 8: Habitability assessment
    planet.habitability = Some(assess_habitability(&planet));

    // Step 9: Timeline construction
    let mut timeline = build_timeline(
        &planet,
        &interior,
        ice_albedo_instability,
        runaway_risk,
        active_plate_tectonics,
    );
    timeline.sort_by(|a, b| a.time_myr.total_cmp(&b.time_myr));

    Ok(PlanetEvolutionOutput {
        planet,
        interior,
        timeline,
    })
}

fn derive_atmosphere(
    planet: &Planet,
    heat: &HeatBudget,
    water_fraction: f64,
) -> AtmosphericProperties {
    let escape_velocity = (2.0 * constants::GRAVITATIONAL_CONSTANT * planet.physical.mass_kg.value
        / planet.physical.radius_m.value)
        .sqrt();
    let retention = ((escape_velocity - 3_000.0) / 8_000.0).clamp(0.0, 1.0);
    let outgassing = (heat.total_heat_w / 4.0e13).clamp(0.0, 3.0);

    // Pressure from outgassing, retention, and water vapor contribution
    let pressure = 101_325.0 * retention * (0.25 + outgassing) * (1.0 + water_fraction * 5.0);
    let density = pressure / (287.0 * 288.0);
    let scale_height = constants::GAS_CONSTANT * 288.0
        / (planet.physical.mass_kg.value * constants::GRAVITATIONAL_CONSTANT
            / planet.physical.radius_m.value.powi(2)
            * 0.02896);

    let mut gases = vec![
        gas("N2", "Nitrogen", 0.70, false),
        gas(
            "CO2",
            "Carbon dioxide",
            (0.02 + outgassing * 0.05).clamp(0.01, 0.30),
            true,
        ),
    ];
    if water_fraction > 0.002 {
        gases.push(gas("H2O", "Water vapour", 0.02, true));
    }
    if retention > 0.6 {
        gases.push(gas("O2", "Oxygen", 0.05, false));
    }
    if retention > 0.4 && planet.physical.mass_kg.value > 0.5 * constants::EARTH_MASS {
        gases.push(gas("Ar", "Argon", 0.009, false));
    }

    let atmosphere_type = if pressure < 1_000.0 {
        AtmosphereType::Trace
    } else if pressure < 50_000.0 {
        AtmosphereType::Thin
    } else if pressure < 250_000.0 {
        AtmosphereType::Standard
    } else {
        AtmosphereType::Dense
    };

    AtmosphericProperties {
        atmosphere_type,
        pressure_pa: Some(measured(
            pressure,
            "Pa",
            "outgassing-retention pressure model",
        )),
        density_kg_m3: Some(measured(density, "kg m^-3", "ideal gas density proxy")),
        scale_height_m: Some(measured(
            scale_height.max(100.0),
            "m",
            "scale-height from gas constant and gravity",
        )),
        layers: vec![AtmosphericLayer {
            name: "Troposphere".to_string(),
            base_altitude_m: measured(0.0, "m", "surface layer base"),
            top_altitude_m: measured(
                scale_height.max(100.0) * 1.5,
                "m",
                "top at 1.5 scale heights",
            ),
            temperature_k: None,
        }],
        composition: gases.clone(),
        cloud_coverage: Some(measured(
            (water_fraction * 6.0).clamp(0.0, 0.85),
            "fraction",
            "water inventory cloud proxy",
        )),
        greenhouse_gases: gases.into_iter().filter(|g| g.is_greenhouse).collect(),
    }
}

fn build_timeline(
    planet: &Planet,
    interior: &InteriorModel,
    ice_albedo_instability: bool,
    runaway_risk: f64,
    active_plate_tectonics: bool,
) -> Vec<EvolutionTimelineEntry> {
    let mut timeline = vec![
        entry(0.0, "Planet forms", "Accretion leaves a differentiated mixture of metal, silicate, volatile, and gas reservoirs."),
        entry(25.0, "Core differentiates", "Dense metal sinks while silicates form a mantle and primitive crust."),
    ];

    if interior.has_liquid_outer_core {
        timeline.push(entry(
            150.0,
            "Magnetic field stabilizes",
            "Liquid conducting core and heat flow support a dynamo.",
        ));
    }

    let volcanism = planet
        .geology
        .as_ref()
        .map(|g| g.volcanism)
        .unwrap_or(worldsmith_models::VolcanicActivity::None);
    if volcanism != worldsmith_models::VolcanicActivity::None {
        timeline.push(entry(
            80.0,
            "Volcanism begins",
            "Internal heat drives mantle melting and volatile outgassing.",
        ));
        timeline.push(entry(
            400.0,
            "Atmosphere thickens",
            "Outgassing supplies secondary atmospheric gases including CO2 and H2O.",
        ));
    }

    if planet.ocean.is_some() {
        timeline.push(entry(
            700.0,
            "Oceans condense",
            "Temperature and pressure permit stable water reservoirs.",
        ));
    }

    if active_plate_tectonics {
        timeline.push(entry(
            500.0,
            "Plate tectonics established",
            "Mantle convection and water lubrication drive plate subduction and crustal recycling.",
        ));
        timeline.push(entry(
            900.0,
            "Carbon-silicate cycle active",
            "Tectonic cycling regulates atmospheric CO2 through subduction and volcanism.",
        ));
    }

    if planet
        .climate
        .as_ref()
        .map(|c| {
            c.climate_type == ClimateType::Temperate || c.climate_type == ClimateType::Tropical
        })
        .unwrap_or(false)
    {
        timeline.push(entry(
            1_200.0,
            "Stable climate established",
            "Surface water, atmosphere, and magnetic shielding remain jointly favorable.",
        ));
    }

    if runaway_risk > 0.5 {
        timeline.push(entry(
            1_500.0,
            "Runaway greenhouse risk",
            "High surface temperature and greenhouse gas concentrations threaten thermal runaway.",
        ));
    }

    if ice_albedo_instability {
        timeline.push(entry(
            1_000.0,
            "Ice-albedo feedback active",
            "Expanding ice coverage reduces absorbed insolation, driving further cooling.",
        ));
    }

    timeline
}

fn gas(formula: &str, name: &str, abundance: f64, is_greenhouse: bool) -> AtmosphericGas {
    AtmosphericGas {
        molecule: Molecule {
            formula: formula.to_string(),
            name: name.to_string(),
            molar_mass_kg_mol: None,
        },
        abundance: measured(
            abundance,
            "mole fraction",
            "outgassing and retention composition proxy",
        ),
        is_greenhouse,
    }
}

fn entry(time_myr: f64, event: &str, explanation: &str) -> EvolutionTimelineEntry {
    EvolutionTimelineEntry {
        time_myr,
        event: event.to_string(),
        explanation: explanation.to_string(),
    }
}

fn measured(value: f64, unit: &str, equation: &str) -> MeasuredValue {
    MeasuredValue {
        value,
        unit: unit.to_string(),
        provenance: Some(worldsmith_models::ScientificProvenance {
            source_equation: Some(equation.to_string()),
            input_variables: Vec::new(),
            confidence: Some(0.58),
            notes: vec!["WorldSmith simplified planetary evolution model".to_string()],
            references: vec![
                "Radiative equilibrium and parameterized terrestrial evolution".to_string(),
            ],
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
                density_kg_m3: Some(measured(5_514.0, "kg m^-3", "Earth density")),
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
    fn earth_like_planet_evolves_successfully() {
        let planet = test_planet();
        let output = evolve_planet(planet, 1.0, 4.5).unwrap();
        assert!(output.planet.geology.is_some());
        assert!(output.planet.atmosphere.is_some());
        assert!(output.planet.climate.is_some());
        assert!(output.planet.magnetic_field.is_some());
        assert!(output.planet.habitability.is_some());
    }

    #[test]
    fn evolution_is_deterministic() {
        let planet = test_planet();
        let a = evolve_planet(planet.clone(), 1.0, 4.5).unwrap();
        let b = evolve_planet(planet, 1.0, 4.5).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn timeline_has_entries() {
        let planet = test_planet();
        let output = evolve_planet(planet, 1.0, 4.5).unwrap();
        assert!(!output.timeline.is_empty());
        assert!(output.timeline.iter().any(|e| e.event == "Planet forms"));
        assert!(output
            .timeline
            .iter()
            .any(|e| e.event == "Core differentiates"));
    }

    #[test]
    fn timeline_is_chronological() {
        let planet = test_planet();
        let output = evolve_planet(planet, 1.0, 4.5).unwrap();
        for window in output.timeline.windows(2) {
            assert!(window[0].time_myr <= window[1].time_myr);
        }
    }

    #[test]
    fn hot_jupiter_shows_runaway_greenhouse() {
        let mut planet = test_planet();
        planet.orbit.semi_major_axis_m = MeasuredValue {
            value: 0.05 * constants::ASTRONOMICAL_UNIT,
            unit: "m".to_string(),
            provenance: None,
        };
        let output = evolve_planet(planet, 1.0, 4.5).unwrap();
        let climate = output.planet.climate.as_ref().unwrap();
        assert_eq!(climate.climate_type, ClimateType::RunawayGreenhouse);
    }

    #[test]
    fn distant_planet_frozen() {
        let mut planet = test_planet();
        planet.orbit.semi_major_axis_m = MeasuredValue {
            value: 5.0 * constants::ASTRONOMICAL_UNIT,
            unit: "m".to_string(),
            provenance: None,
        };
        let output = evolve_planet(planet, 1.0, 4.5).unwrap();
        let climate = output.planet.climate.as_ref().unwrap();
        assert_eq!(climate.climate_type, ClimateType::Frozen);
    }

    #[test]
    fn ocean_planet_produces_water_features() {
        let mut planet = test_planet();
        planet.planet_type = PlanetType::Ocean;
        let output = evolve_planet(planet, 1.0, 4.5).unwrap();
        assert!(output.planet.ocean.is_some());
        let atmosphere = output.planet.atmosphere.as_ref().unwrap();
        assert!(atmosphere
            .composition
            .iter()
            .any(|g| g.molecule.formula == "H2O"));
    }

    #[test]
    fn gas_giant_evolves_with_dense_atmosphere() {
        let mut planet = test_planet();
        planet.planet_type = PlanetType::Gas;
        planet.physical.mass_kg = MeasuredValue {
            value: 100.0 * constants::EARTH_MASS,
            unit: "kg".to_string(),
            provenance: None,
        };
        planet.physical.radius_m = MeasuredValue {
            value: 3.0 * constants::EARTH_RADIUS,
            unit: "m".to_string(),
            provenance: None,
        };
        let output = evolve_planet(planet, 1.0, 4.5).unwrap();
        let atmosphere = output.planet.atmosphere.as_ref().unwrap();
        assert_eq!(atmosphere.atmosphere_type, AtmosphereType::Dense);
    }

    #[test]
    fn habitability_scales_with_conditions() {
        let habitable = test_planet();
        let h = evolve_planet(habitable, 1.0, 4.5).unwrap();

        let mut hostile = test_planet();
        hostile.orbit.semi_major_axis_m = MeasuredValue {
            value: 10.0 * constants::ASTRONOMICAL_UNIT,
            unit: "m".to_string(),
            provenance: None,
        };
        let cold = evolve_planet(hostile, 1.0, 4.5).unwrap();

        assert_ne!(
            h.planet.habitability.as_ref().unwrap().rating,
            cold.planet.habitability.as_ref().unwrap().rating
        );
    }

    #[test]
    fn invalid_inputs_return_error() {
        let planet = test_planet();
        assert!(evolve_planet(planet.clone(), f64::NAN, 4.5).is_err());
        assert!(evolve_planet(planet.clone(), 1.0, -1.0).is_err());
        assert!(evolve_planet(planet, 0.0, 4.5).is_err());
    }

    #[test]
    fn geological_evolution_integrated() {
        let planet = test_planet();
        let output = evolve_planet(planet, 1.0, 4.5).unwrap();
        let geology = output.planet.geology.as_ref().unwrap();
        assert!(geology.heat_flow_w_m2.is_some());
        assert!(geology.plate_system.is_some());
    }

    #[test]
    fn weather_system_derived_from_climate() {
        let planet = test_planet();
        let output = evolve_planet(planet, 1.0, 4.5).unwrap();
        let climate = output.planet.climate.as_ref().unwrap();
        assert!(climate.wind.is_some());
    }
}
