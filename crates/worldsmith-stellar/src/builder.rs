//! Builder API for deterministic stellar profiles.

use serde::{Deserialize, Serialize};
use worldsmith_math::{constants, Vector3};
use worldsmith_models::{MeasuredValue, NamedValue, ScientificProvenance, Star, StarId};

use crate::{
    blackbody::{blackbody_profile, BlackbodyProfile},
    classification::{classify_star, LuminosityClass, SpectralClassification},
    equations,
    errors::{StellarError, StellarResult},
    evolution::{evolution_stage, main_sequence_lifetime_gyr, StellarEvolutionStage},
    habitable_zone::{frost_lines, habitable_zone, FrostLines, HabitableZone},
    radiation::{radiation_profile, stellar_activity, StellarActivity, StellarRadiation},
    validation,
};

/// Complete deterministic stellar calculation result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StellarProfile {
    /// Data-layer star model.
    pub star: Star,
    /// Mass in solar masses.
    pub mass_solar: f64,
    /// Radius in solar radii.
    pub radius_solar: f64,
    /// Luminosity in solar luminosities.
    pub luminosity_solar: f64,
    /// Mean density in kilograms per cubic meter.
    pub density_kg_m3: MeasuredValue,
    /// Escape velocity in meters per second.
    pub escape_velocity_m_s: MeasuredValue,
    /// Stellar flux at 1 AU in watts per square meter.
    pub flux_at_1au_w_m2: MeasuredValue,
    /// Main-sequence lifetime in gigayears.
    pub main_sequence_lifetime_gyr: MeasuredValue,
    /// Simplified evolutionary stage.
    pub evolution_stage: StellarEvolutionStage,
    /// Spectral classification.
    pub classification: SpectralClassification,
    /// Liquid-water habitable zone.
    pub habitable_zone: HabitableZone,
    /// Volatile frost lines.
    pub frost_lines: FrostLines,
    /// Blackbody summary.
    pub blackbody: BlackbodyProfile,
    /// Coarse radiation partition.
    pub radiation: StellarRadiation,
    /// Simplified activity summary.
    pub activity: StellarActivity,
}

/// Builder for validated stellar profiles.
#[derive(Debug, Clone, PartialEq)]
pub struct StarBuilder {
    id: StarId,
    name: String,
    mass_solar: Option<f64>,
    age_gyr: Option<f64>,
    metallicity: Option<f64>,
    rotation_days: Option<f64>,
    position_m: Vector3,
    velocity_m_s: Vector3,
}

impl StarBuilder {
    /// Creates a builder with solar-like defaults for optional identity fields.
    pub fn new() -> Self {
        Self {
            id: StarId(1),
            name: "Unnamed Star".to_string(),
            mass_solar: None,
            age_gyr: Some(0.0),
            metallicity: Some(0.0134),
            rotation_days: None,
            position_m: Vector3::ZERO,
            velocity_m_s: Vector3::ZERO,
        }
    }

    /// Sets the strong star identifier.
    pub fn id(mut self, id: StarId) -> Self {
        self.id = id;
        self
    }

    /// Sets the display name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Sets stellar mass in solar masses.
    pub fn mass_solar(mut self, mass_solar: f64) -> Self {
        self.mass_solar = Some(mass_solar);
        self
    }

    /// Sets stellar age in gigayears.
    pub fn age_gyr(mut self, age_gyr: f64) -> Self {
        self.age_gyr = Some(age_gyr);
        self
    }

    /// Sets mass fraction metallicity.
    pub fn metallicity(mut self, metallicity: f64) -> Self {
        self.metallicity = Some(metallicity);
        self
    }

    /// Sets rotation period in Earth days.
    pub fn rotation_days(mut self, rotation_days: f64) -> Self {
        self.rotation_days = Some(rotation_days);
        self
    }

    /// Sets barycentric position in meters.
    pub fn position_m(mut self, position_m: Vector3) -> Self {
        self.position_m = position_m;
        self
    }

    /// Sets barycentric velocity in meters per second.
    pub fn velocity_m_s(mut self, velocity_m_s: Vector3) -> Self {
        self.velocity_m_s = velocity_m_s;
        self
    }

    /// Validates inputs and builds a deterministic stellar profile.
    pub fn build(self) -> StellarResult<StellarProfile> {
        let mass_solar = self.mass_solar.ok_or_else(|| {
            StellarError::InvalidMass("mass_solar is required before building a star".to_string())
        })?;
        let age_gyr = self.age_gyr.unwrap_or(0.0);
        let metallicity = self.metallicity.unwrap_or(0.0134);
        validation::validate_mass_solar(mass_solar)?;
        validation::validate_age_gyr(age_gyr)?;
        validation::validate_metallicity(metallicity)?;
        validation::validate_rotation_days(self.rotation_days)?;

        let radius_solar = equations::mass_radius_solar(mass_solar);
        let luminosity_solar = equations::mass_luminosity_solar(mass_solar);
        let temperature_k = equations::effective_temperature_k(luminosity_solar, radius_solar);
        if !(2_000.0..=60_000.0).contains(&temperature_k) {
            return Err(StellarError::InvalidTemperature(format!(
                "effective temperature {temperature_k} K is outside supported classification bounds"
            )));
        }

        let mass_kg = mass_solar * constants::SOLAR_MASS;
        let radius_m = radius_solar * constants::SOLAR_RADIUS;
        let luminosity_w = luminosity_solar * constants::SOLAR_LUMINOSITY;
        let surface_gravity = equations::surface_gravity_m_s2(mass_kg, radius_m);
        let density = equations::density_kg_m3(mass_kg, radius_m);
        let escape_velocity = equations::escape_velocity_m_s(mass_kg, radius_m);
        let flux_at_1au = equations::stellar_flux_w_m2(luminosity_w, constants::ASTRONOMICAL_UNIT);
        let lifetime_gyr = main_sequence_lifetime_gyr(mass_solar);
        let stage = evolution_stage(mass_solar, age_gyr);
        let luminosity_class = match stage {
            StellarEvolutionStage::Protostar | StellarEvolutionStage::MainSequence => {
                LuminosityClass::MainSequence
            }
            StellarEvolutionStage::Subgiant => LuminosityClass::Subgiant,
            StellarEvolutionStage::RedGiant => LuminosityClass::Giant,
            StellarEvolutionStage::WhiteDwarf => LuminosityClass::WhiteDwarf,
        };
        let classification = classify_star(temperature_k, luminosity_class);
        let blackbody = blackbody_profile(temperature_k);
        let radiation = radiation_profile(luminosity_w, blackbody);
        let activity = stellar_activity(mass_solar, self.rotation_days);

        let star = Star {
            id: self.id,
            name: self.name,
            spectral_type: classification.spectral_type,
            class: classification.star_class,
            mass_kg: measured(
                mass_kg,
                "kg",
                "M * solar mass",
                vec![("mass_solar", mass_solar, "M_sun")],
            ),
            radius_m: measured(
                radius_m,
                "m",
                "R ~= M^0.8 or M^0.57",
                vec![("mass_solar", mass_solar, "M_sun")],
            ),
            luminosity_w: measured(
                luminosity_w,
                "W",
                "piecewise main-sequence mass-luminosity relation",
                vec![("mass_solar", mass_solar, "M_sun")],
            ),
            effective_temperature_k: measured(
                temperature_k,
                "K",
                "T = T_sun * (L/R^2)^0.25",
                vec![
                    ("luminosity_solar", luminosity_solar, "L_sun"),
                    ("radius_solar", radius_solar, "R_sun"),
                ],
            ),
            surface_gravity_m_s2: measured(
                surface_gravity,
                "m s^-2",
                "g = G M / R^2",
                vec![("mass_kg", mass_kg, "kg"), ("radius_m", radius_m, "m")],
            ),
            metallicity: measured(
                metallicity,
                "mass fraction",
                "builder input",
                vec![("metallicity", metallicity, "mass fraction")],
            ),
            rotation_period_s: self.rotation_days.map(|days| {
                measured(
                    days * 86_400.0,
                    "s",
                    "rotation_days * 86400",
                    vec![("rotation_days", days, "d")],
                )
            }),
            age_s: Some(measured(
                age_gyr * 1.0e9 * constants::JULIAN_YEAR_SECONDS,
                "s",
                "age_gyr * 1e9 Julian years",
                vec![("age_gyr", age_gyr, "Gyr")],
            )),
            position_m: self.position_m,
            velocity_m_s: self.velocity_m_s,
        };

        Ok(StellarProfile {
            star,
            mass_solar,
            radius_solar,
            luminosity_solar,
            density_kg_m3: measured(
                density,
                "kg m^-3",
                "rho = M / (4/3 pi R^3)",
                vec![("mass_kg", mass_kg, "kg"), ("radius_m", radius_m, "m")],
            ),
            escape_velocity_m_s: measured(
                escape_velocity,
                "m s^-1",
                "v_esc = sqrt(2GM/R)",
                vec![("mass_kg", mass_kg, "kg"), ("radius_m", radius_m, "m")],
            ),
            flux_at_1au_w_m2: measured(
                flux_at_1au,
                "W m^-2",
                "F = L / (4 pi d^2)",
                vec![
                    ("luminosity_w", luminosity_w, "W"),
                    ("distance_m", constants::ASTRONOMICAL_UNIT, "m"),
                ],
            ),
            main_sequence_lifetime_gyr: measured(
                lifetime_gyr,
                "Gyr",
                "t_ms ~= 10 Gyr * M/L",
                vec![
                    ("mass_solar", mass_solar, "M_sun"),
                    ("luminosity_solar", luminosity_solar, "L_sun"),
                ],
            ),
            evolution_stage: stage,
            classification,
            habitable_zone: habitable_zone(luminosity_solar),
            frost_lines: frost_lines(luminosity_solar),
            blackbody,
            radiation,
            activity,
        })
    }
}

impl Default for StarBuilder {
    fn default() -> Self {
        Self::new()
    }
}

fn measured(
    value: f64,
    unit: &str,
    equation: &str,
    inputs: Vec<(&str, f64, &str)>,
) -> MeasuredValue {
    MeasuredValue {
        value,
        unit: unit.to_string(),
        provenance: Some(ScientificProvenance {
            source_equation: Some(equation.to_string()),
            input_variables: inputs
                .into_iter()
                .map(|(name, value, unit)| NamedValue {
                    name: name.to_string(),
                    value,
                    unit: Some(unit.to_string()),
                })
                .collect(),
            confidence: Some(0.8),
            notes: vec!["WorldSmith simplified stellar approximation".to_string()],
            references: vec![
                "Stefan-Boltzmann law".to_string(),
                "Piecewise main-sequence mass-luminosity approximation".to_string(),
            ],
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worldsmith_models::SpectralType;

    #[test]
    fn solar_builder_matches_reference_values() {
        let profile = StarBuilder::new()
            .name("Sol")
            .mass_solar(1.0)
            .age_gyr(4.57)
            .metallicity(0.0134)
            .rotation_days(25.4)
            .build()
            .unwrap();
        assert_eq!(profile.star.spectral_type, SpectralType::G);
        assert!((profile.radius_solar - 1.0).abs() < 1e-12);
        assert!((profile.luminosity_solar - 1.0).abs() < 1e-12);
        assert!((profile.star.effective_temperature_k.value - 5_772.0).abs() < 1e-9);
    }

    #[test]
    fn rejects_negative_mass() {
        assert!(StarBuilder::new().mass_solar(-1.0).build().is_err());
    }
}
