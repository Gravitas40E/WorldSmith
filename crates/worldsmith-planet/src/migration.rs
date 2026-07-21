//! Simplified orbital migration model.

use serde::{Deserialize, Serialize};

use crate::{condensation::DiskRegion, embryo::PlanetaryEmbryo};

/// Migration model settings.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MigrationModel {
    /// Strength of gas-driven inward migration.
    pub inward_strength: f64,
    /// Strength of scattering or resonant outward displacement.
    pub outward_strength: f64,
}

impl Default for MigrationModel {
    fn default() -> Self {
        Self {
            inward_strength: 0.015,
            outward_strength: 0.004,
        }
    }
}

/// Migration history entry.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MigrationRecord {
    /// Starting semi-major axis in AU.
    pub from_au: f64,
    /// Final semi-major axis in AU.
    pub to_au: f64,
    /// Net migration in AU.
    pub delta_au: f64,
}

/// Applies simplified migration to an embryo and records the path.
pub fn migrate_embryo(
    embryo: &mut PlanetaryEmbryo,
    gas_fraction: f64,
    model: MigrationModel,
) -> MigrationRecord {
    let from = embryo.orbital_distance_au;
    let mass_factor = (embryo.mass_kg / 5.972_167_9e24).cbrt().clamp(0.1, 10.0);
    let outward = if embryo.formation_region == DiskRegion::OuterDisk {
        model.outward_strength
    } else {
        0.0
    };
    let delta =
        embryo.orbital_distance_au * (outward - model.inward_strength * gas_fraction * mass_factor);
    embryo.orbital_distance_au = (embryo.orbital_distance_au + delta).max(0.03);
    embryo.history.push(format!(
        "Migrated from {:.3} AU to {:.3} AU",
        from, embryo.orbital_distance_au
    ));
    MigrationRecord {
        from_au: from,
        to_au: embryo.orbital_distance_au,
        delta_au: embryo.orbital_distance_au - from,
    }
}
