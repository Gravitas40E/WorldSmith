//! Protoplanetary disk temperature profile.

use worldsmith_math::constants;

/// Estimates disk temperature at orbital distance.
///
/// Uses radiative equilibrium scaling `T ~= 278 K * L^0.25 / sqrt(a_AU)`
/// with a mild early-disk viscous heating floor.
pub fn disk_temperature_k(luminosity_solar: f64, orbital_distance_m: f64, age_myr: f64) -> f64 {
    let au = orbital_distance_m / constants::ASTRONOMICAL_UNIT;
    let irradiation = 278.0 * luminosity_solar.powf(0.25) / au.sqrt();
    let viscous_floor = 150.0 * (-age_myr / 3.0).exp();
    irradiation.max(viscous_floor)
}
