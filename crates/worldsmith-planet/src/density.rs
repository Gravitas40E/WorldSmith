//! Protoplanetary disk surface density and pressure profiles.

use std::f64::consts::PI;

use worldsmith_math::constants;

/// Surface density in kilograms per square meter.
///
/// Uses a minimum-mass-solar-nebula inspired power law `Sigma = Sigma_1AU *
/// (r/AU)^-p`, normalized by total disk mass and outer radius.
pub fn surface_density_kg_m2(
    disk_mass_kg: f64,
    disk_radius_m: f64,
    orbital_distance_m: f64,
    exponent: f64,
) -> f64 {
    let inner_m = 0.05 * constants::ASTRONOMICAL_UNIT;
    let r = orbital_distance_m.max(inner_m);
    let p = exponent;
    let r0 = constants::ASTRONOMICAL_UNIT;
    let norm = if (2.0 - p).abs() < f64::EPSILON {
        2.0 * PI * r0.powf(p) * (disk_radius_m / inner_m).ln()
    } else {
        2.0 * PI * r0.powf(p) * (disk_radius_m.powf(2.0 - p) - inner_m.powf(2.0 - p)) / (2.0 - p)
    };
    let sigma_0 = disk_mass_kg / norm;
    sigma_0 * (r / r0).powf(-p)
}

/// Coarse midplane pressure proxy in pascals.
pub fn midplane_pressure_pa(
    surface_density_kg_m2: f64,
    temperature_k: f64,
    stellar_mass_kg: f64,
    orbital_distance_m: f64,
) -> f64 {
    let orbital_frequency =
        (constants::GRAVITATIONAL_CONSTANT * stellar_mass_kg / orbital_distance_m.powi(3)).sqrt();
    let sound_speed = (1.380_649e-23 * temperature_k / (2.34 * 1.673_557_5e-27)).sqrt();
    let scale_height = sound_speed / orbital_frequency;
    surface_density_kg_m2 * sound_speed.powi(2) / (scale_height * (2.0 * PI).sqrt())
}
