//! Fundamental physical and astronomical constants in SI units unless noted.
//!
//! All values are suitable for planetary-scale simulation. Constants are
//! sourced from CODATA 2018 / IAU 2015 nominal values where applicable.

/// Newtonian gravitational constant (m³ kg⁻¹ s⁻²).
pub const GRAVITATIONAL_CONSTANT: f64 = 6.674_30e-11;

/// Speed of light in vacuum (m s⁻¹).
pub const SPEED_OF_LIGHT: f64 = 299_792_458.0;

/// Stefan–Boltzmann constant (W m⁻² K⁻⁴).
pub const STEFAN_BOLTZMANN: f64 = 5.670_374_419e-8;

/// Astronomical unit (m). IAU 2012 exact definition.
pub const ASTRONOMICAL_UNIT: f64 = 149_597_870_700.0;

/// Solar mass (kg). IAU 2015 nominal.
pub const SOLAR_MASS: f64 = 1.988_409_870e30;

/// Solar radius (m). IAU 2015 nominal.
pub const SOLAR_RADIUS: f64 = 6.957e8;

/// Earth mass (kg). IAU 2015 nominal.
pub const EARTH_MASS: f64 = 5.972_167_9e24;

/// Earth equatorial mean radius (m). IAU 2015 nominal.
pub const EARTH_RADIUS: f64 = 6.3781e6;

/// Standard gravitational acceleration at Earth's surface (m s⁻²).
pub const EARTH_SURFACE_GRAVITY: f64 = 9.806_65;

/// Universal molar gas constant (J mol⁻¹ K⁻¹). CODATA 2018.
pub const GAS_CONSTANT: f64 = 8.314_462_618;

/// Boltzmann constant (J K⁻¹).
pub const BOLTZMANN: f64 = 1.380_649e-23;

/// Standard atmospheric pressure at sea level (Pa).
pub const STANDARD_ATMOSPHERE: f64 = 101_325.0;

/// Julian year in seconds (365.25 days).
pub const JULIAN_YEAR_SECONDS: f64 = 31_557_600.0;

/// Sidereal day in seconds.
pub const SIDEREAL_DAY_SECONDS: f64 = 86_164.090_5;

/// Solar luminosity (W). IAU 2015 nominal.
pub const SOLAR_LUMINOSITY: f64 = 3.828e26;

/// Absolute zero offset for Celsius (K).
pub const CELSIUS_ZERO_KELVIN: f64 = 273.15;

/// Documented metadata for a physical constant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConstantInfo {
    pub name: &'static str,
    pub symbol: &'static str,
    pub value: f64,
    pub unit: &'static str,
    pub description: &'static str,
}

/// Returns documented metadata for all bundled constants.
pub fn all_constants() -> &'static [ConstantInfo] {
    &[
        ConstantInfo {
            name: "Gravitational constant",
            symbol: "G",
            value: GRAVITATIONAL_CONSTANT,
            unit: "m³ kg⁻¹ s⁻²",
            description: "Newtonian constant of gravitation",
        },
        ConstantInfo {
            name: "Speed of light",
            symbol: "c",
            value: SPEED_OF_LIGHT,
            unit: "m s⁻¹",
            description: "Speed of light in vacuum",
        },
        ConstantInfo {
            name: "Stefan–Boltzmann constant",
            symbol: "σ",
            value: STEFAN_BOLTZMANN,
            unit: "W m⁻² K⁻⁴",
            description: "Radiative flux proportionality constant",
        },
        ConstantInfo {
            name: "Astronomical unit",
            symbol: "AU",
            value: ASTRONOMICAL_UNIT,
            unit: "m",
            description: "Mean Earth–Sun distance",
        },
        ConstantInfo {
            name: "Solar mass",
            symbol: "M☉",
            value: SOLAR_MASS,
            unit: "kg",
            description: "IAU nominal solar mass",
        },
        ConstantInfo {
            name: "Earth mass",
            symbol: "M⊕",
            value: EARTH_MASS,
            unit: "kg",
            description: "IAU nominal Earth mass",
        },
        ConstantInfo {
            name: "Earth radius",
            symbol: "R⊕",
            value: EARTH_RADIUS,
            unit: "m",
            description: "IAU nominal Earth equatorial radius",
        },
    ]
}
