//! Spectral and luminosity classification.

use serde::{Deserialize, Serialize};
use worldsmith_models::{SpectralType, StarClass};

/// Morgan-Keenan luminosity class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LuminosityClass {
    /// Supergiant class I.
    Supergiant,
    /// Giant class III.
    Giant,
    /// Subgiant class IV.
    Subgiant,
    /// Main sequence dwarf class V.
    MainSequence,
    /// White dwarf class D.
    WhiteDwarf,
}

impl LuminosityClass {
    /// Returns the compact spectral notation suffix.
    pub fn suffix(self) -> &'static str {
        match self {
            Self::Supergiant => "I",
            Self::Giant => "III",
            Self::Subgiant => "IV",
            Self::MainSequence => "V",
            Self::WhiteDwarf => "D",
        }
    }
}

/// Full spectral classification derived from effective temperature and stage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpectralClassification {
    /// Spectral type letter.
    pub spectral_type: SpectralType,
    /// Numeric subtype from 0 hottest to 9 coolest within a class.
    pub subtype: u8,
    /// Broad stellar data model class.
    pub star_class: StarClass,
    /// Morgan-Keenan luminosity class.
    pub luminosity_class: LuminosityClass,
    /// Human-readable notation such as `G2V`.
    pub notation: String,
}

/// Classifies a star from effective temperature and evolutionary stage.
pub fn classify_star(
    temperature_k: f64,
    luminosity_class: LuminosityClass,
) -> SpectralClassification {
    let (spectral_type, hot, cool) = if temperature_k >= 30_000.0 {
        (SpectralType::O, 50_000.0, 30_000.0)
    } else if temperature_k >= 10_000.0 {
        (SpectralType::B, 30_000.0, 10_000.0)
    } else if temperature_k >= 7_500.0 {
        (SpectralType::A, 10_000.0, 7_500.0)
    } else if temperature_k >= 6_000.0 {
        (SpectralType::F, 7_500.0, 6_000.0)
    } else if temperature_k >= 5_200.0 {
        (SpectralType::G, 6_000.0, 5_200.0)
    } else if temperature_k >= 3_700.0 {
        (SpectralType::K, 5_200.0, 3_700.0)
    } else {
        (SpectralType::M, 3_700.0, 2_400.0)
    };
    let span = hot - cool;
    let subtype = (((hot - temperature_k).clamp(0.0, span) / span) * 10.0)
        .floor()
        .min(9.0) as u8;
    let star_class = match luminosity_class {
        LuminosityClass::Supergiant => StarClass::Supergiant,
        LuminosityClass::Giant => StarClass::Giant,
        LuminosityClass::Subgiant => StarClass::Giant,
        LuminosityClass::MainSequence => StarClass::MainSequence,
        LuminosityClass::WhiteDwarf => StarClass::WhiteDwarf,
    };
    let letter = match spectral_type {
        SpectralType::O => "O",
        SpectralType::B => "B",
        SpectralType::A => "A",
        SpectralType::F => "F",
        SpectralType::G => "G",
        SpectralType::K => "K",
        SpectralType::M => "M",
        _ => "?",
    };
    SpectralClassification {
        spectral_type,
        subtype,
        star_class,
        luminosity_class,
        notation: format!("{letter}{subtype}{}", luminosity_class.suffix()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sun_classifies_as_g_main_sequence() {
        let c = classify_star(5_772.0, LuminosityClass::MainSequence);
        assert_eq!(c.spectral_type, SpectralType::G);
        assert_eq!(c.luminosity_class, LuminosityClass::MainSequence);
    }
}
