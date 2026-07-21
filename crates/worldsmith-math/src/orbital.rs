//! Generic orbital and gravitation utilities.

use std::f64::consts::PI;

use crate::constants::GRAVITATIONAL_CONSTANT;
use crate::Vector3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OrbitalError {
    NonPositiveMass,
    NonPositiveRadius,
    InvalidSemiMajorAxis,
    InvalidEccentricity,
}

pub type OrbitalResult<T> = Result<T, OrbitalError>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OrbitState {
    pub position: Vector3,
    pub velocity: Vector3,
}

#[inline]
pub fn density(mass_kg: f64, radius_m: f64) -> OrbitalResult<f64> {
    if mass_kg <= 0.0 {
        return Err(OrbitalError::NonPositiveMass);
    }
    if radius_m <= 0.0 {
        return Err(OrbitalError::NonPositiveRadius);
    }
    Ok(mass_kg / ((4.0 / 3.0) * PI * radius_m.powi(3)))
}

#[inline]
pub fn surface_gravity(mass_kg: f64, radius_m: f64) -> OrbitalResult<f64> {
    if mass_kg <= 0.0 {
        return Err(OrbitalError::NonPositiveMass);
    }
    if radius_m <= 0.0 {
        return Err(OrbitalError::NonPositiveRadius);
    }
    Ok(GRAVITATIONAL_CONSTANT * mass_kg / radius_m.powi(2))
}

#[inline]
pub fn escape_velocity(mass_kg: f64, radius_m: f64) -> OrbitalResult<f64> {
    if mass_kg <= 0.0 {
        return Err(OrbitalError::NonPositiveMass);
    }
    if radius_m <= 0.0 {
        return Err(OrbitalError::NonPositiveRadius);
    }
    Ok((2.0 * GRAVITATIONAL_CONSTANT * mass_kg / radius_m).sqrt())
}

#[inline]
pub fn circular_orbital_velocity(
    central_mass_kg: f64,
    orbital_radius_m: f64,
) -> OrbitalResult<f64> {
    if central_mass_kg <= 0.0 {
        return Err(OrbitalError::NonPositiveMass);
    }
    if orbital_radius_m <= 0.0 {
        return Err(OrbitalError::NonPositiveRadius);
    }
    Ok((GRAVITATIONAL_CONSTANT * central_mass_kg / orbital_radius_m).sqrt())
}

#[inline]
pub fn kepler_period(central_mass_kg: f64, semi_major_axis_m: f64) -> OrbitalResult<f64> {
    if central_mass_kg <= 0.0 {
        return Err(OrbitalError::NonPositiveMass);
    }
    if semi_major_axis_m <= 0.0 {
        return Err(OrbitalError::InvalidSemiMajorAxis);
    }
    Ok(2.0 * PI * (semi_major_axis_m.powi(3) / (GRAVITATIONAL_CONSTANT * central_mass_kg)).sqrt())
}

#[inline]
pub fn semi_major_axis_from_period(central_mass_kg: f64, period_s: f64) -> OrbitalResult<f64> {
    if central_mass_kg <= 0.0 {
        return Err(OrbitalError::NonPositiveMass);
    }
    if period_s <= 0.0 {
        return Err(OrbitalError::InvalidSemiMajorAxis);
    }
    Ok((GRAVITATIONAL_CONSTANT * central_mass_kg * (period_s / (2.0 * PI)).powi(2)).cbrt())
}

#[inline]
pub fn elliptical_radius(
    semi_major_axis_m: f64,
    eccentricity: f64,
    true_anomaly_radians: f64,
) -> OrbitalResult<f64> {
    if semi_major_axis_m <= 0.0 {
        return Err(OrbitalError::InvalidSemiMajorAxis);
    }
    if !(0.0..1.0).contains(&eccentricity) {
        return Err(OrbitalError::InvalidEccentricity);
    }
    Ok(semi_major_axis_m * (1.0 - eccentricity.powi(2))
        / (1.0 + eccentricity * true_anomaly_radians.cos()))
}

pub fn circular_orbit_state(
    central_mass_kg: f64,
    orbital_radius_m: f64,
    angle_radians: f64,
) -> OrbitalResult<OrbitState> {
    let speed = circular_orbital_velocity(central_mass_kg, orbital_radius_m)?;
    let (s, c) = angle_radians.sin_cos();
    Ok(OrbitState {
        position: Vector3::new(orbital_radius_m * c, orbital_radius_m * s, 0.0),
        velocity: Vector3::new(-speed * s, speed * c, 0.0),
    })
}

pub fn elliptical_orbit_state(
    central_mass_kg: f64,
    semi_major_axis_m: f64,
    eccentricity: f64,
    true_anomaly_radians: f64,
) -> OrbitalResult<OrbitState> {
    if central_mass_kg <= 0.0 {
        return Err(OrbitalError::NonPositiveMass);
    }
    let radius = elliptical_radius(semi_major_axis_m, eccentricity, true_anomaly_radians)?;
    let mu = GRAVITATIONAL_CONSTANT * central_mass_kg;
    let h = (mu * semi_major_axis_m * (1.0 - eccentricity.powi(2))).sqrt();
    let (s, c) = true_anomaly_radians.sin_cos();
    Ok(OrbitState {
        position: Vector3::new(radius * c, radius * s, 0.0),
        velocity: Vector3::new(-mu / h * s, mu / h * (eccentricity + c), 0.0),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{constants, numeric};

    #[test]
    fn earth_surface_gravity_is_close() {
        let g = surface_gravity(constants::EARTH_MASS, constants::EARTH_RADIUS).unwrap();
        assert!(numeric::approx_eq_scaled(g, 9.8, 0.02));
    }

    #[test]
    fn kepler_period_roundtrip() {
        let period = kepler_period(constants::SOLAR_MASS, constants::ASTRONOMICAL_UNIT).unwrap();
        let axis = semi_major_axis_from_period(constants::SOLAR_MASS, period).unwrap();
        assert!(numeric::approx_eq_scaled(
            axis,
            constants::ASTRONOMICAL_UNIT,
            1e-12
        ));
    }
}
