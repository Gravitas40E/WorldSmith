//! Blackbody radiation utilities for stellar effective temperatures.

use serde::{Deserialize, Serialize};
use worldsmith_math::constants;

/// Wien displacement constant in meter kelvin.
pub const WIEN_DISPLACEMENT_M_K: f64 = 2.897_771_955e-3;

/// Approximate visible color name derived from blackbody temperature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApproximateColor {
    /// Red.
    Red,
    /// Orange.
    Orange,
    /// Yellow.
    Yellow,
    /// White-yellow.
    WhiteYellow,
    /// White.
    White,
    /// Blue-white.
    BlueWhite,
    /// Blue.
    Blue,
}

/// RGB color approximation in sRGB byte space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RgbColor {
    /// Red channel.
    pub r: u8,
    /// Green channel.
    pub g: u8,
    /// Blue channel.
    pub b: u8,
}

/// Blackbody-derived radiation summary.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BlackbodyProfile {
    /// Effective temperature in kelvin.
    pub temperature_k: f64,
    /// Peak wavelength in meters from Wien's law.
    pub peak_wavelength_m: f64,
    /// Emitted flux from surface in watts per square meter.
    pub emitted_flux_w_m2: f64,
    /// Approximate human visible color.
    pub approximate_color: ApproximateColor,
    /// Temperature-derived RGB approximation.
    pub rgb: RgbColor,
}

/// Builds a blackbody profile from effective temperature.
pub fn blackbody_profile(temperature_k: f64) -> BlackbodyProfile {
    BlackbodyProfile {
        temperature_k,
        peak_wavelength_m: WIEN_DISPLACEMENT_M_K / temperature_k,
        emitted_flux_w_m2: constants::STEFAN_BOLTZMANN * temperature_k.powi(4),
        approximate_color: approximate_color(temperature_k),
        rgb: temperature_to_rgb(temperature_k),
    }
}

/// Classifies the approximate visible color from blackbody temperature.
pub fn approximate_color(temperature_k: f64) -> ApproximateColor {
    if temperature_k < 3_700.0 {
        ApproximateColor::Red
    } else if temperature_k < 5_000.0 {
        ApproximateColor::Orange
    } else if temperature_k < 5_700.0 {
        ApproximateColor::Yellow
    } else if temperature_k < 6_200.0 {
        ApproximateColor::WhiteYellow
    } else if temperature_k < 7_500.0 {
        ApproximateColor::White
    } else if temperature_k < 10_000.0 {
        ApproximateColor::BlueWhite
    } else {
        ApproximateColor::Blue
    }
}

/// Converts temperature to an approximate sRGB color.
///
/// Uses Tanner Helland's widely used blackbody RGB approximation over the
/// visible stellar temperature range.
pub fn temperature_to_rgb(temperature_k: f64) -> RgbColor {
    let t = (temperature_k / 100.0).clamp(10.0, 400.0);
    let red = if t <= 66.0 {
        255.0
    } else {
        329.698_727_446 * (t - 60.0).powf(-0.133_204_759_2)
    };
    let green = if t <= 66.0 {
        99.470_802_586_1 * t.ln() - 161.119_568_166_1
    } else {
        288.122_169_528_3 * (t - 60.0).powf(-0.075_514_849_2)
    };
    let blue = if t >= 66.0 {
        255.0
    } else if t <= 19.0 {
        0.0
    } else {
        138.517_731_223_1 * (t - 10.0).ln() - 305.044_792_730_7
    };
    RgbColor {
        r: red.clamp(0.0, 255.0).round() as u8,
        g: green.clamp(0.0, 255.0).round() as u8,
        b: blue.clamp(0.0, 255.0).round() as u8,
    }
}
