//! Hydrology and ocean condensation foundations.

use worldsmith_models::{Compound, Molecule, OceanProperties, OceanType};

/// Derives surface water and ocean properties from temperature, pressure, and bulk water.
pub fn derive_ocean_properties(
    surface_temperature_k: f64,
    pressure_pa: f64,
    water_fraction: f64,
) -> Option<OceanProperties> {
    if water_fraction <= 0.001 {
        return None;
    }
    let ocean_type = if (273.15..=373.15).contains(&surface_temperature_k) && pressure_pa > 1_000.0
    {
        OceanType::Water
    } else if surface_temperature_k < 273.15 {
        OceanType::SubsurfaceWater
    } else {
        OceanType::None
    };
    if ocean_type == OceanType::None {
        return None;
    }
    Some(OceanProperties {
        ocean_type,
        coverage: Some(measured(
            (water_fraction * 8.0).clamp(0.05, 0.95),
            "fraction",
            "water inventory and temperature-pressure phase window",
        )),
        average_depth_m: Some(measured(
            (water_fraction * 50_000.0).clamp(10.0, 8_000.0),
            "m",
            "water mass fraction depth proxy",
        )),
        composition: vec![Compound {
            molecule: Molecule {
                formula: "H2O".to_string(),
                name: "Water".to_string(),
                molar_mass_kg_mol: None,
            },
            elements: Vec::new(),
            abundance: None,
        }],
    })
}

fn measured(value: f64, unit: &str, equation: &str) -> worldsmith_models::MeasuredValue {
    worldsmith_models::MeasuredValue {
        value,
        unit: unit.to_string(),
        provenance: Some(worldsmith_models::ScientificProvenance {
            source_equation: Some(equation.to_string()),
            input_variables: Vec::new(),
            confidence: Some(0.55),
            notes: vec!["WorldSmith hydrology approximation".to_string()],
            references: vec!["Water phase constraints from temperature and pressure".to_string()],
        }),
    }
}
