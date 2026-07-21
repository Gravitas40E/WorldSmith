//! Formatted scientific reports for stellar profiles.

use std::fmt::{self, Display, Formatter};

use crate::{blackbody::ApproximateColor, builder::StellarProfile};

/// Human-readable scientific report for a stellar profile.
#[derive(Debug, Clone, PartialEq)]
pub struct StellarReport {
    /// Fully formatted report text.
    pub text: String,
}

impl StellarReport {
    /// Builds a report from a deterministic stellar profile.
    pub fn from_profile(profile: &StellarProfile) -> Self {
        let text = format!(
            "Star Report\n\nName: {}\nMass: {:.3} M_sun\nRadius: {:.3} R_sun\nLuminosity: {:.3} L_sun\nTemperature: {:.0} K\nSpectral Type: {}\nAge: {:.2} Gyr\nMain Sequence Lifetime: {:.2} Gyr\n\nHabitable Zone:\nConservative:\n{:.2} AU - {:.2} AU\n\nOptimistic:\n{:.2} AU - {:.2} AU\n\nFrost Line:\nWater: {:.2} AU\nAmmonia: {:.2} AU\nMethane: {:.2} AU\n\nSurface Gravity:\n{:.0} m/s^2\n\nStellar Flux at 1 AU:\n{:.0} W/m^2\n\nPeak Wavelength:\n{:.0} nm\n\nApproximate Colour:\n{}",
            profile.star.name,
            profile.mass_solar,
            profile.radius_solar,
            profile.luminosity_solar,
            profile.star.effective_temperature_k.value,
            profile.classification.notation,
            profile.star.age_s.as_ref().map(|v| v.value / (1.0e9 * worldsmith_math::constants::JULIAN_YEAR_SECONDS)).unwrap_or(0.0),
            profile.main_sequence_lifetime_gyr.value,
            profile.habitable_zone.conservative_inner_au,
            profile.habitable_zone.conservative_outer_au,
            profile.habitable_zone.optimistic_inner_au,
            profile.habitable_zone.optimistic_outer_au,
            profile.frost_lines.water_au,
            profile.frost_lines.ammonia_au,
            profile.frost_lines.methane_au,
            profile.star.surface_gravity_m_s2.value,
            profile.flux_at_1au_w_m2.value,
            profile.blackbody.peak_wavelength_m * 1.0e9,
            color_name(profile.blackbody.approximate_color),
        );
        Self { text }
    }
}

impl Display for StellarReport {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}

fn color_name(color: ApproximateColor) -> &'static str {
    match color {
        ApproximateColor::Red => "Red",
        ApproximateColor::Orange => "Orange",
        ApproximateColor::Yellow => "Yellow",
        ApproximateColor::WhiteYellow => "White-Yellow",
        ApproximateColor::White => "White",
        ApproximateColor::BlueWhite => "Blue-White",
        ApproximateColor::Blue => "Blue",
    }
}

#[cfg(test)]
mod tests {
    use crate::StarBuilder;

    use super::*;

    #[test]
    fn report_contains_reference_sections() {
        let profile = StarBuilder::new()
            .name("Sol")
            .mass_solar(1.0)
            .age_gyr(4.57)
            .build()
            .unwrap();
        let report = StellarReport::from_profile(&profile);
        assert!(report.text.contains("Star Report"));
        assert!(report.text.contains("Habitable Zone"));
        assert!(report.text.contains("Spectral Type"));
    }
}
