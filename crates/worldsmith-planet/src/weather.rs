//! Weather foundations for planetary evolution.
//!
//! This module models surface weather regimes, wind patterns, precipitation
//! cycles, and storm systems. Weather properties emerge from climate state,
//! ocean coverage, rotation rate, and axial tilt.

use serde::{Deserialize, Serialize};
use worldsmith_models::{MeasuredValue, WeatherType, WindProperties};

use crate::errors::{PlanetFormationError, PlanetFormationResult};

/// Precipitation regime for a weather system.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PrecipitationRegime {
    /// No significant precipitation.
    None,
    /// Episodic low precipitation.
    Arid,
    /// Seasonal precipitation cycles.
    Seasonal,
    /// Persistent precipitation.
    Humid,
    /// Continuous heavy precipitation.
    Torrential,
}

/// Storm intensity classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StormIntensity {
    /// No significant storm activity.
    None,
    /// Mild storms.
    Mild,
    /// Moderate storms.
    Moderate,
    /// Severe storms.
    Severe,
    /// Extreme planetary-scale storms.
    Extreme,
}

/// Weather system model derived from planetary parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeatherSystem {
    /// Precipitation regime.
    pub precipitation: PrecipitationRegime,
    /// Storm intensity.
    pub storm_intensity: StormIntensity,
    /// Annual precipitation in meters per year.
    pub annual_precipitation_m: Option<MeasuredValue>,
    /// Wind properties.
    pub wind: WindProperties,
    /// Weather type suitable for the ClimateProperties model.
    pub weather_type: WeatherType,
}

/// Derives a weather system from planetary climate and rotation parameters.
///
/// # Physics Basis
///
/// - Wind speeds scale with pressure-supported circulation and rotation rate.
/// - Precipitation follows ocean availability, surface temperature, and
///   atmospheric water content.
/// - Storm intensity correlates with available thermal energy and rotation.
pub fn derive_weather_system(
    surface_temperature_k: f64,
    pressure_pa: f64,
    rotation_period_s: Option<f64>,
    has_ocean: bool,
    ice_coverage: f64,
) -> PlanetFormationResult<WeatherSystem> {
    if !surface_temperature_k.is_finite() || surface_temperature_k < 0.0 {
        return Err(PlanetFormationError::InvalidEvolution(
            "surface temperature must be finite and non-negative for weather derivation"
                .to_string(),
        ));
    }

    let rotation_factor = rotation_period_s
        .map(|s| (86_400.0 / s).clamp(0.2, 5.0))
        .unwrap_or(1.0);

    // Wind speed: pressure-supported circulation scaled by rotation
    let base_wind = (pressure_pa / 101_325.0).sqrt().clamp(0.1, 3.0) * 12.0;
    let average_wind_m_s = base_wind * rotation_factor.sqrt();

    // Precipitation from ocean, temperature, and pressure
    let precipitation =
        derive_precipitation(surface_temperature_k, has_ocean, ice_coverage, pressure_pa);

    // Storm intensity from thermal energy reservoir
    let storm_intensity = derive_storm_intensity(surface_temperature_k, rotation_factor, has_ocean);

    let weather_type = classify_weather_type(&precipitation, &storm_intensity, has_ocean);

    Ok(WeatherSystem {
        precipitation,
        storm_intensity,
        annual_precipitation_m: Some(measured(
            compute_annual_precipitation_m(surface_temperature_k, has_ocean, ice_coverage),
            "m yr^-1",
            "annual precipitation from temperature, ocean, and ice coverage",
        )),
        wind: WindProperties {
            average_speed_m_s: Some(measured(
                average_wind_m_s,
                "m s^-1",
                "pressure-supported circulation scaled by rotation",
            )),
            prevailing_direction: Some(worldsmith_math::Vector3::Y),
            weather_type,
        },
        weather_type,
    })
}

fn derive_precipitation(
    surface_temperature_k: f64,
    has_ocean: bool,
    ice_coverage: f64,
    _pressure_pa: f64,
) -> PrecipitationRegime {
    if ice_coverage > 0.8 {
        return PrecipitationRegime::None;
    }
    if !has_ocean || surface_temperature_k < 260.0 {
        return PrecipitationRegime::Arid;
    }
    if surface_temperature_k > 330.0 {
        return PrecipitationRegime::Torrential;
    }
    if surface_temperature_k > 300.0 {
        return PrecipitationRegime::Humid;
    }
    if surface_temperature_k > 273.0 {
        return PrecipitationRegime::Seasonal;
    }
    PrecipitationRegime::Arid
}

fn derive_storm_intensity(
    surface_temperature_k: f64,
    rotation_factor: f64,
    has_ocean: bool,
) -> StormIntensity {
    if !has_ocean || surface_temperature_k < 260.0 {
        return StormIntensity::None;
    }
    let energy = surface_temperature_k - 273.0;
    let intensity = energy * rotation_factor * 0.01;
    if intensity > 2.0 {
        StormIntensity::Extreme
    } else if intensity > 1.2 {
        StormIntensity::Severe
    } else if intensity > 0.6 {
        StormIntensity::Moderate
    } else if intensity > 0.1 {
        StormIntensity::Mild
    } else {
        StormIntensity::None
    }
}

fn classify_weather_type(
    precipitation: &PrecipitationRegime,
    storms: &StormIntensity,
    _has_ocean: bool,
) -> WeatherType {
    match (precipitation, storms) {
        (PrecipitationRegime::Torrential, _) => WeatherType::Precipitating,
        (_, StormIntensity::Extreme | StormIntensity::Severe) => WeatherType::Stormy,
        (PrecipitationRegime::Humid, _) => WeatherType::Precipitating,
        (PrecipitationRegime::Seasonal, _) => WeatherType::Windy,
        (_, StormIntensity::Moderate) => WeatherType::Windy,
        _ => WeatherType::Calm,
    }
}

fn compute_annual_precipitation_m(
    surface_temperature_k: f64,
    has_ocean: bool,
    ice_coverage: f64,
) -> f64 {
    if ice_coverage > 0.8 || !has_ocean {
        return 0.05;
    }
    let thermal = (surface_temperature_k - 273.0).max(0.0);
    let base = thermal * 0.08;
    let ocean_factor = if has_ocean { 1.5 } else { 0.2 };
    let ice_suppression = (1.0 - ice_coverage).max(0.1);
    (base * ocean_factor * ice_suppression).clamp(0.0, 8.0)
}

fn measured(value: f64, unit: &str, equation: &str) -> MeasuredValue {
    MeasuredValue {
        value,
        unit: unit.to_string(),
        provenance: Some(worldsmith_models::ScientificProvenance {
            source_equation: Some(equation.to_string()),
            input_variables: Vec::new(),
            confidence: Some(0.55),
            notes: vec!["WorldSmith weather parameterization".to_string()],
            references: vec!["Simplified terrestrial weather scaling model".to_string()],
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weather_deterministic_from_same_inputs() {
        let a = derive_weather_system(288.0, 101_325.0, Some(86_400.0), true, 0.05).unwrap();
        let b = derive_weather_system(288.0, 101_325.0, Some(86_400.0), true, 0.05).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn arid_planet_has_low_precipitation() {
        let weather = derive_weather_system(250.0, 10_000.0, Some(86_400.0), false, 0.9).unwrap();
        assert_eq!(weather.precipitation, PrecipitationRegime::None);
        assert_eq!(weather.storm_intensity, StormIntensity::None);
    }

    #[test]
    fn ocean_planet_has_higher_precipitation() {
        let arid = derive_weather_system(280.0, 101_325.0, Some(86_400.0), false, 0.05).unwrap();
        let wet = derive_weather_system(280.0, 101_325.0, Some(86_400.0), true, 0.05).unwrap();
        assert_ne!(arid.precipitation, wet.precipitation);
    }

    #[test]
    fn fast_rotation_increases_wind() {
        let slow = derive_weather_system(288.0, 101_325.0, Some(172_800.0), true, 0.05).unwrap();
        let fast = derive_weather_system(288.0, 101_325.0, Some(43_200.0), true, 0.05).unwrap();
        assert!(
            slow.wind.average_speed_m_s.unwrap().value < fast.wind.average_speed_m_s.unwrap().value
        );
    }

    #[test]
    fn hot_ocean_world_produces_storms() {
        // 330K with 1x rotation gives intensity = (330-273) * 1.0 * 0.01 = 0.57 (Mild)
        // Use fast rotation and higher temperature for Moderate+
        let weather = derive_weather_system(330.0, 101_325.0, Some(43_200.0), true, 0.0).unwrap();
        assert!(weather.storm_intensity as u8 >= StormIntensity::Moderate as u8);
    }

    #[test]
    fn frozen_planet_no_storms() {
        let weather = derive_weather_system(200.0, 50_000.0, Some(86_400.0), false, 1.0).unwrap();
        assert_eq!(weather.storm_intensity, StormIntensity::None);
    }

    #[test]
    fn invalid_temperature_returns_error() {
        assert!(derive_weather_system(f64::NAN, 101_325.0, Some(86_400.0), true, 0.05).is_err());
        assert!(derive_weather_system(-10.0, 101_325.0, Some(86_400.0), true, 0.05).is_err());
    }
}
