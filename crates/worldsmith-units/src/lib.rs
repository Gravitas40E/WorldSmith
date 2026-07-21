//! SI unit conversions and dimensional helpers.
//!
//! WorldSmith stores physical quantities in SI units internally. These helpers
//! keep conversion constants centralized and named at API boundaries.

use worldsmith_math::constants;

pub mod length {
    use super::constants;

    pub const METERS_PER_KILOMETER: f64 = 1_000.0;
    pub const METERS_PER_ASTRONOMICAL_UNIT: f64 = constants::ASTRONOMICAL_UNIT;

    #[inline]
    pub fn kilometers_to_meters(kilometers: f64) -> f64 {
        kilometers * METERS_PER_KILOMETER
    }

    #[inline]
    pub fn meters_to_kilometers(meters: f64) -> f64 {
        meters / METERS_PER_KILOMETER
    }

    #[inline]
    pub fn astronomical_units_to_meters(au: f64) -> f64 {
        au * METERS_PER_ASTRONOMICAL_UNIT
    }

    #[inline]
    pub fn meters_to_astronomical_units(meters: f64) -> f64 {
        meters / METERS_PER_ASTRONOMICAL_UNIT
    }

    #[inline]
    pub fn astronomical_units_to_kilometers(au: f64) -> f64 {
        meters_to_kilometers(astronomical_units_to_meters(au))
    }

    #[inline]
    pub fn kilometers_to_astronomical_units(kilometers: f64) -> f64 {
        meters_to_astronomical_units(kilometers_to_meters(kilometers))
    }
}

pub mod mass {
    use super::constants;

    pub const KILOGRAMS_PER_EARTH_MASS: f64 = constants::EARTH_MASS;
    pub const KILOGRAMS_PER_SOLAR_MASS: f64 = constants::SOLAR_MASS;

    #[inline]
    pub fn earth_masses_to_kilograms(earth_masses: f64) -> f64 {
        earth_masses * KILOGRAMS_PER_EARTH_MASS
    }

    #[inline]
    pub fn kilograms_to_earth_masses(kilograms: f64) -> f64 {
        kilograms / KILOGRAMS_PER_EARTH_MASS
    }

    #[inline]
    pub fn solar_masses_to_kilograms(solar_masses: f64) -> f64 {
        solar_masses * KILOGRAMS_PER_SOLAR_MASS
    }

    #[inline]
    pub fn kilograms_to_solar_masses(kilograms: f64) -> f64 {
        kilograms / KILOGRAMS_PER_SOLAR_MASS
    }
}

pub mod temperature {
    use super::constants;

    #[inline]
    pub fn celsius_to_kelvin(celsius: f64) -> f64 {
        celsius + constants::CELSIUS_ZERO_KELVIN
    }

    #[inline]
    pub fn kelvin_to_celsius(kelvin: f64) -> f64 {
        kelvin - constants::CELSIUS_ZERO_KELVIN
    }
}

pub mod time {
    pub const SECONDS_PER_MINUTE: f64 = 60.0;
    pub const SECONDS_PER_HOUR: f64 = 3_600.0;
    pub const SECONDS_PER_DAY: f64 = 86_400.0;
    pub const SECONDS_PER_JULIAN_YEAR: f64 = worldsmith_math::constants::JULIAN_YEAR_SECONDS;

    #[inline]
    pub fn minutes_to_seconds(minutes: f64) -> f64 {
        minutes * SECONDS_PER_MINUTE
    }

    #[inline]
    pub fn hours_to_seconds(hours: f64) -> f64 {
        hours * SECONDS_PER_HOUR
    }

    #[inline]
    pub fn days_to_seconds(days: f64) -> f64 {
        days * SECONDS_PER_DAY
    }

    #[inline]
    pub fn seconds_to_days(seconds: f64) -> f64 {
        seconds / SECONDS_PER_DAY
    }

    #[inline]
    pub fn julian_years_to_seconds(years: f64) -> f64 {
        years * SECONDS_PER_JULIAN_YEAR
    }

    #[inline]
    pub fn seconds_to_julian_years(seconds: f64) -> f64 {
        seconds / SECONDS_PER_JULIAN_YEAR
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worldsmith_math::numeric;

    #[test]
    fn length_roundtrip() {
        let km = 149_597_870.7;
        let au = length::kilometers_to_astronomical_units(km);
        assert!(numeric::approx_eq_scaled(au, 1.0, 1e-12));
    }

    #[test]
    fn mass_roundtrip() {
        let kg = mass::earth_masses_to_kilograms(2.0);
        assert_eq!(mass::kilograms_to_earth_masses(kg), 2.0);
    }

    #[test]
    fn temperature_conversion() {
        assert_eq!(temperature::celsius_to_kelvin(0.0), 273.15);
        assert_eq!(temperature::kelvin_to_celsius(273.15), 0.0);
    }

    #[test]
    fn days_to_seconds() {
        assert_eq!(time::days_to_seconds(2.0), 172_800.0);
    }
}
