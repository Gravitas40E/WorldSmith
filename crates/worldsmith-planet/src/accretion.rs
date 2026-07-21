//! Simplified deterministic accretion model.

use serde::{Deserialize, Serialize};

use crate::{
    embryo::{PlanetaryEmbryo, PlanetaryEmbryoComposition},
    planetesimal::Planetesimal,
};

/// Collision/accretion history entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccretionEvent {
    /// Embryo receiving material.
    pub embryo_id: u64,
    /// Accreted planetesimal id.
    pub body_id: u64,
    /// Mass added in kilograms.
    pub added_mass_kg: f64,
    /// Orbital distance separation in AU.
    pub separation_au: f64,
}

/// Result of accretion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccretionSummary {
    /// Produced planetary embryos.
    pub embryos: Vec<PlanetaryEmbryo>,
    /// Collision events.
    pub collisions: Vec<AccretionEvent>,
    /// Minor accretion event count.
    pub minor_accretion_events: u32,
}

/// Accretes planetesimals into embryos using feeding zones in orbital space.
pub fn accrete_planetesimals(
    mut bodies: Vec<Planetesimal>,
    promotion_mass_kg: f64,
) -> AccretionSummary {
    bodies.sort_by(|a, b| a.orbital_distance_au.total_cmp(&b.orbital_distance_au));
    let mut embryos: Vec<PlanetaryEmbryo> = Vec::new();
    let mut collisions = Vec::new();
    let mut minor = 0;

    for body in &bodies {
        let feeding_zone_au = 0.08 * body.orbital_distance_au.sqrt().max(0.5);
        if let Some(embryo) = embryos.iter_mut().find(|embryo| {
            (embryo.orbital_distance_au - body.orbital_distance_au).abs() <= feeding_zone_au
        }) {
            let previous_mass = embryo.mass_kg;
            let total_mass = previous_mass + body.mass_kg;
            embryo.composition = mix_composition(
                embryo.composition,
                previous_mass,
                PlanetaryEmbryo::from_planetesimal(0, body).composition,
                body.mass_kg,
            );
            embryo.mass_kg = total_mass;
            embryo.orbital_distance_au = (embryo.orbital_distance_au * previous_mass
                + body.orbital_distance_au * body.mass_kg)
                / total_mass;
            embryo.parent_bodies.push(body.id);
            embryo.recompute_radius();
            embryo
                .history
                .push(format!("Accreted planetesimal {}", body.id));
            collisions.push(AccretionEvent {
                embryo_id: embryo.id,
                body_id: body.id,
                added_mass_kg: body.mass_kg,
                separation_au: (embryo.orbital_distance_au - body.orbital_distance_au).abs(),
            });
            if body.mass_kg < promotion_mass_kg {
                minor += 1;
            }
        } else if body.mass_kg >= promotion_mass_kg || embryos.is_empty() {
            embryos.push(PlanetaryEmbryo::from_planetesimal(
                embryos.len() as u64 + 1,
                body,
            ));
        } else {
            minor += 1;
        }
    }

    embryos.retain(|embryo| embryo.mass_kg >= promotion_mass_kg);
    AccretionSummary {
        embryos,
        collisions,
        minor_accretion_events: minor,
    }
}

fn mix_composition(
    a: PlanetaryEmbryoComposition,
    mass_a: f64,
    b: PlanetaryEmbryoComposition,
    mass_b: f64,
) -> PlanetaryEmbryoComposition {
    let total = mass_a + mass_b;
    PlanetaryEmbryoComposition {
        metal_fraction: (a.metal_fraction * mass_a + b.metal_fraction * mass_b) / total,
        silicate_fraction: (a.silicate_fraction * mass_a + b.silicate_fraction * mass_b) / total,
        water_fraction: (a.water_fraction * mass_a + b.water_fraction * mass_b) / total,
        ice_fraction: (a.ice_fraction * mass_a + b.ice_fraction * mass_b) / total,
        gas_fraction: (a.gas_fraction * mass_a + b.gas_fraction * mass_b) / total,
    }
}
