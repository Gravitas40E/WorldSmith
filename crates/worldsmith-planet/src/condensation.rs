//! Condensation sequence and disk regions.

use serde::{Deserialize, Serialize};
use worldsmith_models::SurfaceMaterial;

/// Named protoplanetary disk region.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiskRegion {
    /// Hot inner disk dominated by metals and refractory silicates.
    InnerDisk,
    /// Region around liquid-water habitable zone.
    HabitableRegion,
    /// Region around or beyond the water frost line.
    FrostLine,
    /// Cold outer disk rich in volatile ices.
    OuterDisk,
}

/// Material that can condense at a location in the disk.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CondensedMaterial {
    /// Surface/material category for future composition models.
    pub material: SurfaceMaterial,
    /// Condensation temperature in kelvin.
    pub condensation_temperature_k: f64,
    /// Relative abundance weight available at the location.
    pub relative_abundance: f64,
}

/// Returns available condensed materials for a disk temperature.
pub fn available_materials(temperature_k: f64) -> Vec<CondensedMaterial> {
    let mut materials = Vec::new();
    if temperature_k <= 1_350.0 {
        materials.push(CondensedMaterial {
            material: SurfaceMaterial::Metal,
            condensation_temperature_k: 1_350.0,
            relative_abundance: 0.33,
        });
    }
    if temperature_k <= 1_200.0 {
        materials.push(CondensedMaterial {
            material: SurfaceMaterial::SilicateRock,
            condensation_temperature_k: 1_200.0,
            relative_abundance: 0.67,
        });
    }
    if temperature_k <= 170.0 {
        materials.push(CondensedMaterial {
            material: SurfaceMaterial::WaterIce,
            condensation_temperature_k: 170.0,
            relative_abundance: 0.50,
        });
    }
    if temperature_k <= 80.0 {
        materials.push(CondensedMaterial {
            material: SurfaceMaterial::CarbonDioxideIce,
            condensation_temperature_k: 80.0,
            relative_abundance: 0.15,
        });
    }
    if temperature_k <= 30.0 {
        materials.push(CondensedMaterial {
            material: SurfaceMaterial::Organics,
            condensation_temperature_k: 30.0,
            relative_abundance: 0.10,
        });
    }
    materials
}

/// Classifies a disk region from orbital distance and key boundaries.
pub fn classify_region(
    orbital_au: f64,
    habitable_inner_au: f64,
    habitable_outer_au: f64,
    water_frost_au: f64,
) -> DiskRegion {
    if orbital_au >= water_frost_au * 1.25 {
        DiskRegion::OuterDisk
    } else if orbital_au >= water_frost_au * 0.85 {
        DiskRegion::FrostLine
    } else if (habitable_inner_au..=habitable_outer_au).contains(&orbital_au) {
        DiskRegion::HabitableRegion
    } else {
        DiskRegion::InnerDisk
    }
}
