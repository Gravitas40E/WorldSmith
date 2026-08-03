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

/// Computes the mean anomaly at an absolute simulation time.
///
/// Mean anomaly grows linearly with time about the orbit focus:
///
/// ```text
/// M(t) = M₀ + 2π · (t / T)
/// ```
///
/// where `M₀` is the mean anomaly at `t = 0` and `T` is the orbital period.
/// The returned value is wrapped to `[0, 2π)` so downstream anomaly solvers
/// can consume it directly.
///
/// ## Arguments
/// - `timestamp_s` — absolute simulation time in seconds.
/// - `period_s` — orbital period in seconds. Must be strictly positive.
/// - `initial_mean_anomaly_radians` — mean anomaly at simulation epoch.
///   `None` is equivalent to `0.0`, placing periapsis at `t = 0`.
///
/// # Errors
/// Returns `Err(OrbitalError::InvalidSemiMajorAxis)` if `period_s <= 0.0`.
#[inline]
pub fn mean_anomaly_from_time(
    timestamp_s: f64,
    period_s: f64,
    initial_mean_anomaly_radians: Option<f64>,
) -> OrbitalResult<f64> {
    if period_s <= 0.0 {
        return Err(OrbitalError::InvalidSemiMajorAxis);
    }
    let two_pi = 2.0 * PI;
    let m0 = initial_mean_anomaly_radians.unwrap_or(0.0);
    let m = m0 + two_pi * (timestamp_s / period_s);
    let mut m = m % two_pi;
    if m < 0.0 {
        m += two_pi;
    }
    Ok(m)
}

/// Solves Kepler's Equation for eccentric anomaly `E` given mean anomaly `M`
/// and eccentricity `e`.
///
/// Kepler's Equation:
///
/// ```text
/// f(E) = E - e·sin(E) - M = 0
/// ```
///
/// Solved by Newton-Raphson iteration:
///
/// ```text
/// E_{n+1} = E_n - f(E_n) / f'(E_n)
/// f'(E)   = 1 - e·cos(E)
/// ```
///
/// ## Convergence
/// - **Initial guess:** `E₀ = M` when `e < 0.8`; `E₀ = π` otherwise. The
///   high-eccentricity branch avoids the near-zero derivative region near
///   `E = 0` where Newton-Raphson diverges for `e ≈ 1`.
/// - **Stopping criteria:** `|f(E)| < tolerance` or `iterations >= max_iterations`.
/// - For double-precision `f64` inputs and non-extreme eccentricities,
///   convergence typically occurs within 3–5 iterations.
///
/// ## Arguments
/// - `mean_anomaly` — mean anomaly `M` in radians.
/// - `eccentricity` — orbital eccentricity. The valid range is `[0, 1)`.
///   Behaviour at `e = 1` is not special-cased.
/// - `tolerance` — convergence threshold on `|f(E)|`. Recommended `1e-12`.
/// - `max_iterations` — hard cap on Newton steps. Recommended ≥ 20.
///
/// # Safety
/// This function never returns NaN. If the iteration limit is reached before
/// convergence, the last iterate is returned unchanged.
///
/// # Panics
/// Does not panic.
///
/// # References
/// - Meeus, J. *Astronomical Algorithms*, 2nd ed., ch. 30.
/// - Murray, C. D. & Dermott, S. F. *Solar System Dynamics*, ch. 2.
#[inline]
pub fn eccentric_anomaly_from_mean(
    mean_anomaly: f64,
    eccentricity: f64,
    tolerance: f64,
    max_iterations: u32,
) -> f64 {
    if eccentricity == 0.0 {
        // Circular orbits have E = M exactly. Skip Newton machinery.
        return mean_anomaly;
    }
    let e = eccentricity;
    let mut e_anom = if e < 0.8 { mean_anomaly } else { PI };
    for _ in 0..max_iterations {
        let f = e_anom - e * e_anom.sin() - mean_anomaly;
        if f.abs() < tolerance {
            return e_anom;
        }
        let fp = 1.0 - e * e_anom.cos();
        if fp == 0.0 {
            return e_anom;
        }
        e_anom -= f / fp;
    }
    // Return best estimate after max iterations; still very close for valid inputs.
    e_anom
}

/// Converts eccentric anomaly `E` to true anomaly `ν` for eccentricity `e`.
///
/// Uses the numerically stable `atan2` formulation (Meeus ch. 30):
///
/// ```text
/// ν = 2 · atan2( √(1+e) · sin(E/2), √(1-e) · cos(E/2) )
/// ```
///
/// This formulation remains stable at periapsis (`E → 0`) and apoapsis
/// (`E → π`) where simpler `cos ν` / `sin ν` formulas suffer from
/// catastrophic cancellation.
///
/// For circular orbits (`e = 0`), true anomaly equals eccentric anomaly
/// exactly; this function returns `E` without calling `atan2`.
///
/// The result is normalized to `[0, 2π)`.
///
/// # References
/// - Meeus, J. *Astronomical Algorithms*, 2nd ed., ch. 30.
#[inline]
pub fn true_anomaly_from_eccentric(eccentricity: f64, eccentric_anomaly: f64) -> f64 {
    if eccentricity == 0.0 {
        let mut nu = eccentric_anomaly % (2.0 * PI);
        if nu < 0.0 {
            nu += 2.0 * PI;
        }
        return nu;
    }
    let s = (eccentric_anomaly / 2.0).sin();
    let c = (eccentric_anomaly / 2.0).cos();
    let nu = 2.0 * ((1.0 + eccentricity).sqrt() * s).atan2((1.0 - eccentricity).sqrt() * c);
    let two_pi = 2.0 * PI;
    let mut nu = nu % two_pi;
    if nu < 0.0 {
        nu += two_pi;
    }
    nu
}

/// Propagates a Keplerian orbit from an absolute simulation timestamp to its
/// parent-relative position and velocity vectors.
///
/// This is a pure function: it performs no engine, `WorldState`, or RNG
/// access. The returned `OrbitState` is expressed in the **parent-relative**
/// frame; callers must add the parent body's world-space position to obtain
/// absolute barycentric coordinates.
///
/// ## Algorithm
/// 1. Compute mean anomaly `M(t) = M₀ + 2π·(t/T)`.
/// 2. Solve Kepler's equation for eccentric anomaly `E`.
/// 3. Convert `E` to true anomaly `ν`.
/// 4. Build orbital-plane state (`z = 0`).
/// 5. Rotate by inclination about the X-axis.
///
/// ## Performance
/// No heap allocation. Single Newton-Raphson loop per call.
///
/// # Errors
/// Returns `Err` for physically invalid inputs (non-positive mass or
/// semi-major axis, or eccentricity outside `[0, 1)`).
///
/// # References
/// - Meeus, J. *Astronomical Algorithms*, 2nd ed., chs. 30–33.
/// - Murray, C. D. & Dermott, S. F. *Solar System Dynamics*, ch. 2.
pub fn propagate_orbit_state(
    central_mass_kg: f64,
    semi_major_axis_m: f64,
    eccentricity: f64,
    inclination_rad: f64,
    period_s: f64,
    timestamp_s: f64,
    phase_offset_radians: Option<f64>,
) -> OrbitalResult<OrbitState> {
    if central_mass_kg <= 0.0 {
        return Err(OrbitalError::NonPositiveMass);
    }
    if semi_major_axis_m <= 0.0 {
        return Err(OrbitalError::InvalidSemiMajorAxis);
    }
    if !(0.0..1.0).contains(&eccentricity) {
        return Err(OrbitalError::InvalidEccentricity);
    }
    if period_s <= 0.0 {
        return Err(OrbitalError::InvalidSemiMajorAxis);
    }

    let mean_anomaly = mean_anomaly_from_time(timestamp_s, period_s, phase_offset_radians)?;
    let eccentric_anomaly = eccentric_anomaly_from_mean(mean_anomaly, eccentricity, 1e-12, 50);
    let true_anomaly = true_anomaly_from_eccentric(eccentricity, eccentric_anomaly);

    let mut orbit = elliptical_orbit_state(
        central_mass_kg,
        semi_major_axis_m,
        eccentricity,
        true_anomaly,
    )?;

    let cos_i = inclination_rad.cos();
    let sin_i = inclination_rad.sin();
    let y = orbit.position.y;
    orbit.position.y = y * cos_i;
    orbit.position.z = y * sin_i;

    let vy = orbit.velocity.y;
    orbit.velocity.y = vy * cos_i;
    orbit.velocity.z = vy * sin_i;

    Ok(orbit)
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

    // --- mean_anomaly_from_time tests ---

    #[test]
    fn mean_anomaly_at_half_period_is_pi() {
        let period = constants::JULIAN_YEAR_SECONDS;
        let m = mean_anomaly_from_time(period / 2.0, period, None).unwrap();
        assert!(numeric::approx_eq(m, PI, 1e-12));
    }

    #[test]
    fn mean_anomaly_at_zero_is_zero() {
        let m = mean_anomaly_from_time(0.0, 1.0, None).unwrap();
        assert!(numeric::approx_eq(m, 0.0, 1e-12));
    }

    #[test]
    fn mean_anomaly_full_period_wraps_to_zero() {
        let period = 100.0;
        let m = mean_anomaly_from_time(period, period, None).unwrap();
        assert!(numeric::approx_eq(m, 0.0, 1e-12));
    }

    #[test]
    fn mean_anomaly_negative_timestamp() {
        let period = 100.0;
        let m = mean_anomaly_from_time(-25.0, period, None).unwrap();
        assert!(numeric::approx_eq(m, 3.0 * PI / 2.0, 1e-12));
    }

    #[test]
    fn mean_anomaly_large_timestamp() {
        let period = 10.0;
        let m = mean_anomaly_from_time(1e15, period, None).unwrap();
        assert!((0.0..2.0 * PI).contains(&m));
    }

    #[test]
    fn mean_anomaly_with_initial_offset() {
        let m = mean_anomaly_from_time(0.0, 10.0, Some(1.0)).unwrap();
        assert!(numeric::approx_eq(m, 1.0, 1e-12));
    }

    #[test]
    fn mean_anomaly_zero_or_negative_period_errors() {
        assert!(mean_anomaly_from_time(1.0, 0.0, None).is_err());
        assert!(mean_anomaly_from_time(1.0, -1.0, None).is_err());
    }

    // --- eccentric_anomaly_from_mean tests ---

    #[test]
    fn eccentric_anomaly_circular_equals_mean() {
        let candidates = [0.0, 0.5, 1.2345, PI, 4.0, 5.5, 2.0 * PI];
        for m in candidates.iter() {
            let e_anom = eccentric_anomaly_from_mean(*m, 0.0, 1e-12, 20);
            assert!(
                numeric::approx_eq(e_anom, *m, 1e-12),
                "M={}, E={}",
                m,
                e_anom
            );
        }
    }

    #[test]
    fn eccentric_anomaly_elliptical_converges() {
        let cases = [(0.5, 0.1), (PI, 0.3), (4.0, 0.6), (5.5, 0.9)];
        for (m, e) in cases.iter() {
            let e_anom = eccentric_anomaly_from_mean(*m, *e, 1e-12, 50);
            let residual = e_anom - e * e_anom.sin() - m;
            assert!(
                residual.abs() < 1e-12,
                "e={} m={} E={} residual={}",
                e,
                m,
                e_anom,
                residual
            );
            assert!(e_anom.is_finite());
        }
    }

    #[test]
    fn eccentric_anomaly_high_eccentricity_boundary() {
        for m in [0.0, PI] {
            let e_anom = eccentric_anomaly_from_mean(m, 0.99, 1e-12, 50);
            let residual = e_anom - 0.99 * e_anom.sin() - m;
            assert!(residual.abs() < 1e-12);
            assert!(e_anom.is_finite());
        }
    }

    #[test]
    fn eccentric_anomaly_extreme_eccentricity_near_parabolic() {
        let e = 0.999999;
        let m = 0.1;
        let e_anom = eccentric_anomaly_from_mean(m, e, 1e-10, 100);
        let residual = e_anom - e * e_anom.sin() - m;
        assert!(residual.abs() < 1e-10);
        assert!(e_anom.is_finite());
    }

    // --- true_anomaly_from_eccentric tests ---

    #[test]
    fn true_anomaly_circular_equals_eccentric() {
        for e_anom in [0.0, 0.5, 1.2345, PI, 4.0] {
            let nu = true_anomaly_from_eccentric(0.0, e_anom);
            assert!(numeric::approx_eq(nu, e_anom, 1e-12));
            assert!((0.0..2.0 * PI).contains(&nu));
        }
    }

    #[test]
    fn true_anomaly_elliptical_finite_and_normalized() {
        let e = 0.5;
        for e_anom in [0.0, PI / 2.0, PI, 3.0 * PI / 2.0, 2.0 * PI] {
            let nu = true_anomaly_from_eccentric(e, e_anom);
            assert!(nu.is_finite());
            assert!((0.0..2.0 * PI).contains(&nu));
        }
    }

    #[test]
    fn true_anomaly_monotonic_with_eccentric_anomaly() {
        let e = 0.7;
        let mut prev = true_anomaly_from_eccentric(e, 0.0);
        for i in 1..20 {
            let e_anom = (i as f64) * PI / 10.0;
            let nu = true_anomaly_from_eccentric(e, e_anom);
            assert!(nu >= prev, "nu({})={} < prev={}", e_anom, nu, prev);
            prev = nu;
        }
    }

    #[test]
    fn circular_propagate_matches_circular_orbit_state() {
        let mass = constants::SOLAR_MASS;
        let a = constants::ASTRONOMICAL_UNIT;
        let period = kepler_period(mass, a).unwrap();
        for ts in [0.0, period * 0.25, period * 0.5, period * 0.75] {
            let propagated = propagate_orbit_state(mass, a, 0.0, 0.0, period, ts, None).unwrap();
            let expected_angle = 2.0 * PI * (ts / period);
            let direct = circular_orbit_state(mass, a, expected_angle).unwrap();
            assert!(numeric::approx_eq_scaled(
                propagated.position.x,
                direct.position.x,
                1e-9
            ));
            assert!(numeric::approx_eq_scaled(
                propagated.position.y,
                direct.position.y,
                1e-9
            ));
            assert!(numeric::approx_eq_scaled(propagated.position.z, 0.0, 1e-12));
            assert!(numeric::approx_eq_scaled(
                propagated.velocity.x,
                direct.velocity.x,
                1e-9
            ));
            assert!(numeric::approx_eq_scaled(
                propagated.velocity.y,
                direct.velocity.y,
                1e-9
            ));
            assert!(numeric::approx_eq_scaled(propagated.velocity.z, 0.0, 1e-12));
        }
        // Closure after one period: propagate_orbit_state normalizes mean anomaly
        // to 0.0 at t = T, so position must match t = 0 exactly.
        let state0 = propagate_orbit_state(mass, a, 0.0, 0.0, period, 0.0, None).unwrap();
        let state_t = propagate_orbit_state(mass, a, 0.0, 0.0, period, period, None).unwrap();
        assert!(state0.position.distance(state_t.position) < 1e-9);
        assert!(state0.velocity.distance(state_t.velocity) < 1e-9);
    }

    #[test]
    fn elliptical_propagate_closure_after_one_period() {
        let mass = constants::SOLAR_MASS;
        let a = 2.0 * constants::ASTRONOMICAL_UNIT;
        let e = 0.3;
        let period = kepler_period(mass, a).unwrap();
        let state0 = propagate_orbit_state(mass, a, e, 0.0, period, 0.0, None).unwrap();
        let state1 = propagate_orbit_state(mass, a, e, 0.0, period, period, None).unwrap();
        assert!(state0.position.distance(state1.position) < 1e-6);
        assert!(state0.velocity.distance(state1.velocity) < 1e-6);
    }

    #[test]
    fn elliptical_propagate_periapsis_radius() {
        let mass = constants::SOLAR_MASS;
        let a = 2.0 * constants::ASTRONOMICAL_UNIT;
        let e = 0.25;
        let period = kepler_period(mass, a).unwrap();
        let state = propagate_orbit_state(mass, a, e, 0.0, period, 0.0, None).unwrap();
        let expected_periapsis = a * (1.0 - e);
        let r = state.position.magnitude();
        assert!(numeric::approx_eq_scaled(r, expected_periapsis, 1e-6));
    }

    #[test]
    fn elliptical_propagate_apoapsis_radius() {
        let mass = constants::SOLAR_MASS;
        let a = 2.0 * constants::ASTRONOMICAL_UNIT;
        let e = 0.25;
        let period = kepler_period(mass, a).unwrap();
        let state = propagate_orbit_state(mass, a, e, 0.0, period, period / 2.0, None).unwrap();
        let expected_apoapsis = a * (1.0 + e);
        let r = state.position.magnitude();
        assert!(numeric::approx_eq_scaled(r, expected_apoapsis, 1e-6));
    }

    #[test]
    fn propagate_inclination_rotates_plane() {
        let mass = constants::SOLAR_MASS;
        let a = constants::ASTRONOMICAL_UNIT;
        let e = 0.0;
        let period = kepler_period(mass, a).unwrap();
        let ts = 0.25 * period;
        let inclined = propagate_orbit_state(mass, a, e, 0.5, period, ts, None).unwrap();
        let reference = propagate_orbit_state(mass, a, e, 0.0, period, ts, None).unwrap();
        assert!(numeric::approx_eq_scaled(
            inclined.position.magnitude(),
            reference.position.magnitude(),
            1e-9
        ));
        assert!(inclined.position.z.abs() > 1e-3);
    }

    #[test]
    fn propagate_velocity_highest_at_periapsis() {
        let mass = constants::SOLAR_MASS;
        let a = 2.0 * constants::ASTRONOMICAL_UNIT;
        let e = 0.3;
        let period = kepler_period(mass, a).unwrap();
        let v_peri = propagate_orbit_state(mass, a, e, 0.0, period, 0.0, None)
            .unwrap()
            .velocity
            .magnitude();
        let v_apo = propagate_orbit_state(mass, a, e, 0.0, period, period / 2.0, None)
            .unwrap()
            .velocity
            .magnitude();
        assert!(v_peri > v_apo);
        let mu = GRAVITATIONAL_CONSTANT * mass;
        let r_peri = a * (1.0 - e);
        let r_apo = a * (1.0 + e);
        let v_peri_vis = (mu * (2.0 / r_peri - 1.0 / a)).sqrt();
        let v_apo_vis = (mu * (2.0 / r_apo - 1.0 / a)).sqrt();
        assert!(numeric::approx_eq_scaled(v_peri, v_peri_vis, 1e-6));
        assert!(numeric::approx_eq_scaled(v_apo, v_apo_vis, 1e-6));
    }

    #[test]
    fn propagate_energy_conserved_over_orbit() {
        let mass = constants::SOLAR_MASS;
        let a = constants::ASTRONOMICAL_UNIT;
        let e = 0.8;
        let period = kepler_period(mass, a).unwrap();
        let mu = GRAVITATIONAL_CONSTANT * mass;
        let mut energies = Vec::new();
        for i in 0..1000 {
            let ts = (i as f64) * period / 1000.0;
            let state = propagate_orbit_state(mass, a, e, 0.3, period, ts, None).unwrap();
            let r = state.position.magnitude();
            let v = state.velocity.magnitude();
            energies.push(0.5 * v * v - mu / r);
        }
        let mean = energies.iter().sum::<f64>() / energies.len() as f64;
        let max_dev = energies
            .iter()
            .map(|&e| (e - mean).abs())
            .fold(0.0_f64, f64::max);
        assert!(max_dev < 1e-3, "max energy deviation: {}", max_dev);
    }

    #[test]
    fn propagate_extreme_semi_major_axis_stable() {
        let mass = constants::SOLAR_MASS;
        let a = 1e18;
        let e = 0.01;
        let period = kepler_period(mass, a).unwrap();
        let state = propagate_orbit_state(mass, a, e, 0.0, period, 1e12, None).unwrap();
        assert!(state.position.x.is_finite());
        assert!(state.velocity.x.is_finite());
    }

    #[test]
    fn propagate_near_circular_high_precision() {
        let mass = constants::SOLAR_MASS;
        let a = constants::ASTRONOMICAL_UNIT;
        let e = 1e-8;
        let period = kepler_period(mass, a).unwrap();
        for ts in [0.0, period * 0.25, period * 0.5, period * 0.75, period] {
            let state = propagate_orbit_state(mass, a, e, 0.0, period, ts, None).unwrap();
            let r = state.position.magnitude();
            assert!(numeric::approx_eq_scaled(r, a, 1e-6));
        }
    }

    #[test]
    fn propagate_invalid_inputs_return_error() {
        assert!(propagate_orbit_state(0.0, 1.0, 0.0, 0.0, 1.0, 0.0, None).is_err());
        assert!(propagate_orbit_state(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, None).is_err());
        assert!(propagate_orbit_state(1.0, 1.0, 1.0, 0.0, 1.0, 0.0, None).is_err());
        assert!(propagate_orbit_state(1.0, 1.0, 0.0, 0.0, 0.0, 0.0, None).is_err());
    }
}
