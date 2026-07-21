//! Explainable habitability assessment.

use worldsmith_models::{HabitabilityProperties, HabitabilityRating, LifeStage, Planet};

/// Assesses habitability from evolved planet state.
pub fn assess_habitability(planet: &Planet) -> HabitabilityProperties {
    let mut positives = Vec::new();
    let mut negatives = Vec::new();
    let mut score = 0.0;

    if let Some(climate) = &planet.climate {
        if let Some(temp) = &climate.average_temperature_k {
            if (273.15..=323.15).contains(&temp.value) {
                score += 0.30;
                positives.push("Surface temperature supports liquid water".to_string());
            } else {
                negatives.push(
                    "Average surface temperature is outside the liquid-water comfort range"
                        .to_string(),
                );
            }
        }
    }
    if planet.ocean.is_some() {
        score += 0.25;
        positives.push("Stable water reservoir is present".to_string());
    } else {
        negatives.push("No stable ocean or hydrosphere was inferred".to_string());
    }
    if let Some(atmosphere) = &planet.atmosphere {
        if atmosphere
            .pressure_pa
            .as_ref()
            .map(|p| (10_000.0..=500_000.0).contains(&p.value))
            .unwrap_or(false)
        {
            score += 0.20;
            positives.push("Atmospheric pressure is compatible with surface liquids".to_string());
        } else {
            negatives.push("Atmospheric pressure is hostile or uncertain".to_string());
        }
    }
    if let Some(field) = &planet.magnetic_field {
        if field
            .field_strength_t
            .as_ref()
            .map(|f| f.value > 5.0e-6)
            .unwrap_or(false)
        {
            score += 0.15;
            positives.push("Magnetic field can reduce atmospheric erosion".to_string());
        }
    }
    if planet
        .physical
        .surface_gravity_m_s2
        .as_ref()
        .map(|g| (3.0..=25.0).contains(&g.value))
        .unwrap_or(false)
    {
        score += 0.10;
        positives
            .push("Surface gravity is within a broad terrestrial habitability range".to_string());
    } else {
        negatives.push("Surface gravity may be too low or too high".to_string());
    }

    let rating = if score >= 0.75 {
        HabitabilityRating::Favorable
    } else if score >= 0.50 {
        HabitabilityRating::Potential
    } else if score >= 0.25 {
        HabitabilityRating::Marginal
    } else {
        HabitabilityRating::Hostile
    };

    HabitabilityProperties {
        rating,
        positive_factors: positives,
        negative_factors: negatives,
        scientific_notes: vec![format!(
            "Habitability score derived from additive physical constraints: {:.2}",
            score
        )],
        confidence: Some(0.55),
        life_stage: LifeStage::None,
    }
}
