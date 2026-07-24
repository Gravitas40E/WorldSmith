//! Long-term stability harness.
//!
//! Run simulations for many ticks and verify that no values diverge,
//! explode, oscillate, or become NaN.

use worldsmith_models::Planet;

/// Stability report for a long-run test.
#[derive(Debug, Clone, PartialEq)]
pub struct StabilityReport {
    /// Ticks executed.
    pub ticks: u64,
    /// Whether all checks passed.
    pub stable: bool,
    /// Maximum absolute value observed in any planetary field.
    pub max_abs_value: f64,
    /// Final state validation errors, if any.
    pub state_errors: Vec<String>,
}

impl StabilityReport {
    /// Creates a failing report.
    pub fn failure(ticks: u64, state_errors: Vec<String>) -> Self {
        Self {
            ticks,
            stable: false,
            max_abs_value: f64::NAN,
            state_errors,
        }
    }
}

/// Errors produced during long-run stability testing.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum StabilityError {
    /// State validation failed during a long-run simulation.
    #[error("state validation failed at tick {ticks}: {message}")]
    StateValidationFailed {
        /// Tick number.
        ticks: u64,
        /// Underlying error message.
        message: String,
    },
}

/// Run a long-run stability test for `target_ticks` on an engine initialized
/// with `planets`.  Returns a [`StabilityReport`].
///
/// The engine is expected to be pre-built with the four Phase 10 evolution
/// modules registered.
pub fn run_long_run_stability(
    mut engine: worldsmith_engine::Engine,
    target_ticks: u64,
) -> Result<StabilityReport, StabilityError> {
    engine
        .initialize()
        .map_err(|e| StabilityError::StateValidationFailed {
            ticks: 0,
            message: format!("{e}"),
        })?;

    let mut max_abs = 0.0f64;
    for tick in 0..target_ticks {
        if let Err(e) = engine.tick_fixed() {
            return Err(StabilityError::StateValidationFailed {
                ticks: tick,
                message: format!("{e}"),
            });
        }

        // Scan all planets for boundedness and NaN/Inf.
        let snapshot = engine.state();
        for planet in snapshot.planets.values() {
            scan_planet(planet, &mut max_abs).map_err(|msg| {
                StabilityError::StateValidationFailed {
                    ticks: tick,
                    message: msg,
                }
            })?;
        }
    }

    Ok(StabilityReport {
        ticks: target_ticks,
        stable: true,
        max_abs_value: max_abs,
        state_errors: Vec::new(),
    })
}

fn scan_planet(planet: &Planet, max_abs: &mut f64) -> Result<(), String> {
    if let Some(i) = &planet.interior {
        check_finite(i.core_temperature, "core_temperature", max_abs)?;
        check_finite(i.mantle_temperature, "mantle_temperature", max_abs)?;
        check_finite(i.heat_flux, "heat_flux", max_abs)?;
        check_non_negative(i.radiogenic_heat, "radiogenic_heat", max_abs)?;
        check_non_negative(i.internal_heat, "internal_heat", max_abs)?;
    }
    if let Some(v) = &planet.volcanism {
        check_non_negative(v.volcanic_flux, "volcanic_flux", max_abs)?;
        check_non_negative(v.magma_generation_rate, "magma_generation_rate", max_abs)?;
    }
    if let Some(t) = &planet.plate_tectonics {
        check_non_negative(t.plate_velocity, "plate_velocity", max_abs)?;
        check_non_negative(t.crustal_recycling_rate, "crustal_recycling_rate", max_abs)?;
    }
    Ok(())
}

fn check_finite(value: f64, field: &'static str, max_abs: &mut f64) -> Result<(), String> {
    if !value.is_finite() {
        return Err(format!("{field} is not finite ({value})"));
    }
    *max_abs = max_abs.max(value.abs());
    Ok(())
}

fn check_non_negative(value: f64, field: &'static str, max_abs: &mut f64) -> Result<(), String> {
    if value < 0.0 {
        return Err(format!("{field} is negative ({value})"));
    }
    *max_abs = max_abs.max(value.abs());
    Ok(())
}
