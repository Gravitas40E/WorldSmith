//! Planetary embryo data produced by simplified accretion.

use serde::{Deserialize, Serialize};
use worldsmith_models::SurfaceMaterial;

use crate::{condensation::DiskRegion, planetesimal::Planetesimal};

/// Bulk composition fractions for a planetary embryo.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct PlanetaryEmbryoComposition {
    /// Metal core-forming fraction.
    pub metal_fraction: f64,
    /// Silicate fraction.
    pub silicate_fraction: f64,
    /// Water fraction.
    pub water_fraction: f64,
    /// Other volatile ice fraction.
    pub ice_fraction: f64,
    /// Gas envelope fraction.
    pub gas_fraction: f64,
}

/// Growing protoplanet created from merged planetesimals.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanetaryEmbryo {
    /// Deterministic local identifier.
    pub id: u64,
    /// Mass in kilograms.
    pub mass_kg: f64,
    /// Radius in meters.
    pub radius_m: f64,
    /// Orbital distance in AU.
    pub orbital_distance_au: f64,
    /// Bulk composition fractions.
    pub composition: PlanetaryEmbryoComposition,
    /// Formation region.
    pub formation_region: DiskRegion,
    /// Parent planetesimal identifiers.
    pub parent_bodies: Vec<u64>,
    /// Formation history notes.
    pub history: Vec<String>,
}

impl PlanetaryEmbryo {
    /// Promotes a planetesimal into an embryo seed.
    pub fn from_planetesimal(id: u64, body: &Planetesimal) -> Self {
        let composition = composition_from_body(body);
        Self {
            id,
            mass_kg: body.mass_kg,
            radius_m: body.radius_m,
            orbital_distance_au: body.orbital_distance_au,
            composition,
            formation_region: body.formation_region,
            parent_bodies: vec![body.id],
            history: vec![format!("Seeded from planetesimal {}", body.id)],
        }
    }

    /// Recomputes radius from mass and composition-derived density.
    pub fn recompute_radius(&mut self) {
        let density = bulk_density_kg_m3(self.composition);
        self.radius_m = (3.0 * self.mass_kg / (4.0 * std::f64::consts::PI * density)).cbrt();
    }
}

/// Estimates bulk density from composition.
pub fn bulk_density_kg_m3(composition: PlanetaryEmbryoComposition) -> f64 {
    let rock = composition.silicate_fraction * 3_300.0;
    let metal = composition.metal_fraction * 7_800.0;
    let water = composition.water_fraction * 1_000.0;
    let ice = composition.ice_fraction * 900.0;
    let gas = composition.gas_fraction * 250.0;
    (rock + metal + water + ice + gas).max(500.0)
}

fn composition_from_body(body: &Planetesimal) -> PlanetaryEmbryoComposition {
    let mut composition = PlanetaryEmbryoComposition::default();
    for material in &body.composition {
        match material.surface_material {
            SurfaceMaterial::Metal => composition.metal_fraction += 0.30,
            SurfaceMaterial::SilicateRock => composition.silicate_fraction += 0.55,
            SurfaceMaterial::WaterIce | SurfaceMaterial::LiquidWater => {
                composition.water_fraction += 0.25
            }
            SurfaceMaterial::CarbonDioxideIce | SurfaceMaterial::Organics => {
                composition.ice_fraction += 0.10
            }
            _ => composition.silicate_fraction += 0.05,
        }
    }
    let total = composition.metal_fraction
        + composition.silicate_fraction
        + composition.water_fraction
        + composition.ice_fraction
        + composition.gas_fraction;
    if total <= 0.0 {
        composition.silicate_fraction = 0.67;
        composition.metal_fraction = 0.33;
        return composition;
    }
    composition.metal_fraction /= total;
    composition.silicate_fraction /= total;
    composition.water_fraction /= total;
    composition.ice_fraction /= total;
    composition.gas_fraction /= total;
    composition
}
