//! Scientific reports for planet formation and evolution outputs.

use std::fmt::{self, Display, Formatter};

use worldsmith_math::constants;

use crate::{
    builder::PlanetFormationOutput, embryo::bulk_density_kg_m3, evolution::PlanetEvolutionOutput,
};

/// Human-readable report explaining a planet's formation.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanetFormationReport {
    /// Formatted report text.
    pub text: String,
}

impl PlanetFormationReport {
    /// Builds a report for a planet by index in a formation result.
    pub fn from_result(result: &PlanetFormationOutput, planet_index: usize) -> Option<Self> {
        let planet = result.planets.get(planet_index)?;
        let embryo = result.accretion.embryos.get(planet_index)?;
        let density_g_cm3 = bulk_density_kg_m3(embryo.composition) / 1_000.0;
        let earth_mass = planet.physical.mass_kg.value / constants::EARTH_MASS;
        let earth_radius = planet.physical.radius_m.value / constants::EARTH_RADIUS;
        let major = result
            .accretion
            .collisions
            .iter()
            .filter(|event| event.embryo_id == embryo.id)
            .count();
        let text = format!(
            "Planet Report\n\nMass:\n{:.2} Earth\n\nRadius:\n{:.2} Earth\n\nDensity:\n{:.2} g/cm^3\n\nGravity:\n{:.1} m/s^2\n\nCore Fraction:\n{:.0}%\n\nWater:\n{:.1}%\n\nOrbit:\n{:.2} AU\n\nFormation Region:\n{:?}\n\nMigration:\n{:.3} AU\n\nClassification:\n{:?}\n\nPrimary Materials:\nMetal\nSilicates\nIces as available\n\nFormation History:\n{} major collisions\n{} minor accretion events\n{}",
            earth_mass,
            earth_radius,
            density_g_cm3,
            planet.physical.surface_gravity_m_s2.as_ref().map(|v| v.value).unwrap_or(0.0),
            embryo.composition.metal_fraction * 100.0,
            embryo.composition.water_fraction * 100.0,
            embryo.orbital_distance_au,
            embryo.formation_region,
            result.migration_records.get(planet_index).map(|record| record.delta_au).unwrap_or(0.0),
            planet.class,
            major,
            result.accretion.minor_accretion_events,
            embryo.history.join("\n"),
        );
        Some(Self { text })
    }
}

impl Display for PlanetFormationReport {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}

/// Complete evolved-planet scientific report.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanetEvolutionReport {
    /// Formatted report text.
    pub text: String,
}

impl PlanetEvolutionReport {
    /// Builds a report from a planetary evolution output.
    pub fn from_output(output: &PlanetEvolutionOutput) -> Self {
        let planet = &output.planet;

        // Weather details
        let weather_desc = planet
            .climate
            .as_ref()
            .and_then(|c| {
                c.wind.as_ref().map(|w| {
                    format!(
                        "Wind speed: {:.1} m/s\nWeather: {:?}",
                        w.average_speed_m_s.as_ref().map(|v| v.value).unwrap_or(0.0),
                        w.weather_type
                    )
                })
            })
            .unwrap_or_else(|| "No weather data".to_string());

        // Geological details
        let geology_desc = planet
            .geology
            .as_ref()
            .map(|g| {
                format!(
                    "Volcanism: {:?}\nTectonics: {:?}\nHeat flow: {:.3} W/m^2",
                    g.volcanism,
                    g.plate_system
                        .as_ref()
                        .map(|p| p.activity)
                        .unwrap_or(worldsmith_models::TectonicActivity::None),
                    g.heat_flow_w_m2.as_ref().map(|v| v.value).unwrap_or(0.0),
                )
            })
            .unwrap_or_else(|| "No geology data".to_string());

        // Climate details
        let climate_desc = planet
            .climate
            .as_ref()
            .map(|c| {
                format!(
                "Type: {:?}\nAverage temperature: {:.1} K\nIce coverage: {:.0}%\nHumidity: {:.0}%",
                c.climate_type,
                c.average_temperature_k.as_ref().map(|v| v.value).unwrap_or(0.0),
                c.ice_coverage.as_ref().map(|v| v.value * 100.0).unwrap_or(0.0),
                c.humidity.as_ref().map(|v| v.value * 100.0).unwrap_or(0.0),
            )
            })
            .unwrap_or_else(|| "No climate data".to_string());

        // Atmosphere details
        let atmosphere_desc = planet
            .atmosphere
            .as_ref()
            .map(|a| {
                let gases: Vec<String> = a
                    .composition
                    .iter()
                    .map(|g| format!("{}: {:.1}%", g.molecule.formula, g.abundance.value * 100.0))
                    .collect();
                format!(
                    "Pressure: {:.0} Pa\nType: {:?}\nGases:\n  {}",
                    a.pressure_pa.as_ref().map(|v| v.value).unwrap_or(0.0),
                    a.atmosphere_type,
                    gases.join("\n  "),
                )
            })
            .unwrap_or_else(|| "No atmosphere data".to_string());

        // Magnetic field details
        let magnetic_desc = planet
            .magnetic_field
            .as_ref()
            .map(|m| {
                format!(
                    "Strength: {:.2} uT\nMagnetosphere radius: {:.0} km",
                    m.field_strength_t
                        .as_ref()
                        .map(|v| v.value * 1.0e6)
                        .unwrap_or(0.0),
                    m.magnetosphere_radius_m
                        .as_ref()
                        .map(|v| v.value / 1_000.0)
                        .unwrap_or(0.0),
                )
            })
            .unwrap_or_else(|| "No magnetic field data".to_string());

        let text = format!(
            "Planet Evolution Report\n\n\
            --- Interior ---\n\
            Core radius: {:.0} km\n\
            Mantle thickness: {:.0} km\n\
            Crust thickness: {:.1} km\n\
            Internal temperature: {:.0} K\n\
            Cooling rate: {:.1} K/Gyr\n\n\
            --- Geology ---\n\
            {geology_desc}\n\n\
            --- Magnetic Field ---\n\
            {magnetic_desc}\n\n\
            --- Atmosphere ---\n\
            {atmosphere_desc}\n\n\
            --- Climate ---\n\
            {climate_desc}\n\n\
            --- Weather ---\n\
            {weather_desc}\n\n\
            --- Hydrology ---\n\
            Ocean: {}\n\n\
            --- Habitability ---\n\
            Rating: {:?}\n\
            Confidence: {:.2}\n\
            Positive factors:\n  {}\n\
            Negative factors:\n  {}\n\n\
            --- Evolution Timeline ---\n\
            {}",
            output.interior.core_radius_m / 1_000.0,
            output.interior.mantle_thickness_m / 1_000.0,
            output.interior.crust_thickness_m / 1_000.0,
            output.interior.heat_budget.internal_temperature_k,
            output.interior.heat_budget.cooling_rate_k_gyr,
            if planet.ocean.is_some() {
                "present"
            } else {
                "absent"
            },
            planet
                .habitability
                .as_ref()
                .map(|h| h.rating)
                .unwrap_or(worldsmith_models::HabitabilityRating::Unknown),
            planet
                .habitability
                .as_ref()
                .and_then(|h| h.confidence)
                .unwrap_or(0.0),
            planet
                .habitability
                .as_ref()
                .map(|h| h.positive_factors.join("\n  "))
                .unwrap_or_default(),
            planet
                .habitability
                .as_ref()
                .map(|h| h.negative_factors.join("\n  "))
                .unwrap_or_default(),
            output
                .timeline
                .iter()
                .map(|e| format!("{:.0} Myr - {}: {}", e.time_myr, e.event, e.explanation))
                .collect::<Vec<_>>()
                .join("\n"),
        );
        Self { text }
    }
}

impl Display for PlanetEvolutionReport {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}
