//! Habitable zone and volatile frost-line approximations.
//!
//! Habitable zone flux limits use common Kopparapu et al.-style solar effective
//! flux constants with distance scaled by `sqrt(L/L_sun)`.

use serde::{Deserialize, Serialize};

/// Conservative and optimistic liquid-water habitable zone bounds in AU.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HabitableZone {
    /// Runaway greenhouse inner edge in AU.
    pub conservative_inner_au: f64,
    /// Maximum greenhouse outer edge in AU.
    pub conservative_outer_au: f64,
    /// Recent Venus optimistic inner edge in AU.
    pub optimistic_inner_au: f64,
    /// Early Mars optimistic outer edge in AU.
    pub optimistic_outer_au: f64,
}

/// Frost lines for volatile condensation in AU.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FrostLines {
    /// Water snow line in AU.
    pub water_au: f64,
    /// Ammonia frost line in AU.
    pub ammonia_au: f64,
    /// Methane frost line in AU.
    pub methane_au: f64,
}

/// Calculates habitable zone bounds from luminosity in solar units.
pub fn habitable_zone(luminosity_solar: f64) -> HabitableZone {
    let root = luminosity_solar.sqrt();
    HabitableZone {
        conservative_inner_au: root / 1.107_f64.sqrt(),
        conservative_outer_au: root / 0.356_f64.sqrt(),
        optimistic_inner_au: root / 1.776_f64.sqrt(),
        optimistic_outer_au: root / 0.320_f64.sqrt(),
    }
}

/// Calculates frost lines from luminosity in solar units.
///
/// Uses radiative equilibrium scaling `d ~= (278 K / T_condensation)^2 sqrt(L)`.
pub fn frost_lines(luminosity_solar: f64) -> FrostLines {
    let root = luminosity_solar.sqrt();
    FrostLines {
        water_au: root * (278.0_f64 / 170.0_f64).powi(2),
        ammonia_au: root * (278.0_f64 / 80.0_f64).powi(2),
        methane_au: root * (278.0_f64 / 30.0_f64).powi(2),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solar_habitable_zone_is_reference_like() {
        let hz = habitable_zone(1.0);
        assert!((hz.conservative_inner_au - 0.95).abs() < 0.02);
        assert!((hz.conservative_outer_au - 1.67).abs() < 0.02);
    }
}
