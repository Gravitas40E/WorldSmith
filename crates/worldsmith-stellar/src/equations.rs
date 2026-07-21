//! Astrophysical approximation equations.
//!
//! The relations here are deterministic, documented approximations intended for
//! scientific simulation scaffolding. They are not replacements for full stellar
//! structure models.

use std::f64::consts::PI;

use worldsmith_math::constants;

/// Nominal solar effective temperature in kelvin.
pub const SOLAR_EFFECTIVE_TEMPERATURE_K: f64 = 5_772.0;

/// Estimates main-sequence radius from mass in solar units.
///
/// Approximation: low-mass stars use roughly `R ~ M^0.8`; solar and massive
/// main-sequence stars use a shallower `R ~ M^0.57` relation commonly used in
/// introductory stellar astrophysics.
pub fn mass_radius_solar(mass_solar: f64) -> f64 {
    if mass_solar < 1.0 {
        mass_solar.powf(0.8)
    } else {
        mass_solar.powf(0.57)
    }
}

/// Estimates luminosity from mass in solar units.
///
/// Piecewise mass-luminosity relation after common Eker/Duric-style
/// approximations. Output is in solar luminosities.
pub fn mass_luminosity_solar(mass_solar: f64) -> f64 {
    if mass_solar < 0.43 {
        0.23 * mass_solar.powf(2.3)
    } else if mass_solar < 2.0 {
        mass_solar.powf(4.0)
    } else if mass_solar < 20.0 {
        1.5 * mass_solar.powf(3.5)
    } else {
        3_200.0 * mass_solar
    }
}

/// Derives effective temperature from luminosity and radius.
///
/// From Stefan-Boltzmann scaling: `T = T_sun * (L/R^2)^0.25`.
pub fn effective_temperature_k(luminosity_solar: f64, radius_solar: f64) -> f64 {
    SOLAR_EFFECTIVE_TEMPERATURE_K * (luminosity_solar / radius_solar.powi(2)).powf(0.25)
}

/// Computes surface gravity in meters per second squared.
pub fn surface_gravity_m_s2(mass_kg: f64, radius_m: f64) -> f64 {
    constants::GRAVITATIONAL_CONSTANT * mass_kg / radius_m.powi(2)
}

/// Computes mean density in kilograms per cubic meter.
pub fn density_kg_m3(mass_kg: f64, radius_m: f64) -> f64 {
    mass_kg / ((4.0 / 3.0) * PI * radius_m.powi(3))
}

/// Computes escape velocity in meters per second.
pub fn escape_velocity_m_s(mass_kg: f64, radius_m: f64) -> f64 {
    (2.0 * constants::GRAVITATIONAL_CONSTANT * mass_kg / radius_m).sqrt()
}

/// Computes stellar flux at orbital distance in meters.
pub fn stellar_flux_w_m2(luminosity_w: f64, distance_m: f64) -> f64 {
    luminosity_w / (4.0 * PI * distance_m.powi(2))
}

/// Computes bolometric emitted power from radius and effective temperature.
pub fn bolometric_luminosity_w(radius_m: f64, temperature_k: f64) -> f64 {
    4.0 * PI * radius_m.powi(2) * constants::STEFAN_BOLTZMANN * temperature_k.powi(4)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solar_scaling_matches_sun() {
        assert!((mass_radius_solar(1.0) - 1.0).abs() < 1e-12);
        assert!((mass_luminosity_solar(1.0) - 1.0).abs() < 1e-12);
        assert!((effective_temperature_k(1.0, 1.0) - SOLAR_EFFECTIVE_TEMPERATURE_K).abs() < 1e-12);
    }
}
