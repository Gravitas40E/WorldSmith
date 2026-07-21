//! Stellar radiation and activity data structures.

use serde::{Deserialize, Serialize};

use crate::blackbody::BlackbodyProfile;

/// Coarse stellar radiation partition.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StellarRadiation {
    /// Ultraviolet luminosity fraction.
    pub ultraviolet_fraction: f64,
    /// Visible luminosity fraction.
    pub visible_fraction: f64,
    /// Infrared luminosity fraction.
    pub infrared_fraction: f64,
    /// Ultraviolet output in watts.
    pub ultraviolet_w: f64,
    /// Visible output in watts.
    pub visible_w: f64,
    /// Infrared output in watts.
    pub infrared_w: f64,
}

/// Simplified stellar activity summary for future magnetic and flare models.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StellarActivity {
    /// Dimensionless magnetic activity level in `[0, 1]`.
    pub magnetic_activity: f64,
    /// Solar wind mass-loss proxy in solar-relative units.
    pub solar_wind_relative: f64,
    /// Dimensionless flare activity level in `[0, 1]`.
    pub flare_activity: f64,
}

/// Estimates broad UV/visible/IR luminosity bands from blackbody temperature.
///
/// This is an intentionally coarse partition. Future phases can replace it
/// with spectral integration without changing the public data shape.
pub fn radiation_profile(luminosity_w: f64, blackbody: BlackbodyProfile) -> StellarRadiation {
    let t = blackbody.temperature_k;
    let ultraviolet_fraction = ((t - 5_500.0) / 20_000.0).clamp(0.01, 0.45);
    let infrared_fraction = ((6_500.0 - t) / 6_500.0).clamp(0.15, 0.80);
    let visible_fraction = (1.0 - ultraviolet_fraction - infrared_fraction).clamp(0.05, 0.80);
    let total = ultraviolet_fraction + visible_fraction + infrared_fraction;
    let uv = ultraviolet_fraction / total;
    let visible = visible_fraction / total;
    let ir = infrared_fraction / total;
    StellarRadiation {
        ultraviolet_fraction: uv,
        visible_fraction: visible,
        infrared_fraction: ir,
        ultraviolet_w: luminosity_w * uv,
        visible_w: luminosity_w * visible,
        infrared_w: luminosity_w * ir,
    }
}

/// Estimates simplified stellar activity from mass and rotation.
pub fn stellar_activity(mass_solar: f64, rotation_days: Option<f64>) -> StellarActivity {
    let rotation_factor = rotation_days
        .map(|days| (25.4 / days).clamp(0.0, 5.0))
        .unwrap_or(1.0);
    let magnetic = (0.15 * rotation_factor * mass_solar.powf(-0.3)).clamp(0.0, 1.0);
    StellarActivity {
        magnetic_activity: magnetic,
        solar_wind_relative: (rotation_factor * mass_solar).clamp(0.05, 10.0),
        flare_activity: (magnetic * 0.8).clamp(0.0, 1.0),
    }
}
