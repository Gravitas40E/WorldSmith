//! Deterministic planetesimal population generation.

use serde::{Deserialize, Serialize};
use worldsmith_math::{constants, Vector3};
use worldsmith_models::Material;
use worldsmith_rng::RngStream;

use crate::{
    condensation::{available_materials, classify_region, DiskRegion},
    disk::{disk_regions, ProtoplanetaryDisk},
};

/// Small body formed from condensed disk solids.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Planetesimal {
    /// Deterministic local identifier.
    pub id: u64,
    /// Mass in kilograms.
    pub mass_kg: f64,
    /// Radius in meters.
    pub radius_m: f64,
    /// Orbital distance in AU.
    pub orbital_distance_au: f64,
    /// Approximate orbital velocity vector in meters per second.
    pub velocity_m_s: Vector3,
    /// Condensed composition available at formation.
    pub composition: Vec<Material>,
    /// Disk region where this body formed.
    pub formation_region: DiskRegion,
}

/// Generates deterministic planetesimals from disk surface-density structure.
pub fn generate_planetesimals(
    disk: &ProtoplanetaryDisk,
    count: usize,
    rng: &mut RngStream,
) -> Vec<Planetesimal> {
    let bounds = disk_regions(disk.stellar_luminosity_solar);
    let mut bodies = Vec::with_capacity(count);
    let inner_au = 0.3_f64;
    let outer_au = (disk.disk_radius_m / constants::ASTRONOMICAL_UNIT).min(50.0);
    let stellar_mass_kg = disk.stellar_mass_solar * constants::SOLAR_MASS;

    for i in 0..count {
        let t = (i as f64 + 0.5) / count.max(1) as f64;
        let jitter = rng.next_f64_range(-0.015, 0.015);
        let orbital_au = inner_au * (outer_au / inner_au).powf((t + jitter).clamp(0.0, 1.0));
        let orbital_m = orbital_au * constants::ASTRONOMICAL_UNIT;
        let sigma = disk.surface_density_kg_m2(orbital_m) * disk.dust_fraction;
        let annulus_width_m = 0.03 * orbital_m;
        let feeding_zone = 2.0 * std::f64::consts::PI * orbital_m * annulus_width_m;
        let mass_kg = (sigma * feeding_zone * rng.next_f64_range(0.2, 1.2)).max(1.0e15);
        let density = if orbital_au >= bounds.water_frost_au {
            1_800.0
        } else {
            3_500.0
        };
        let radius_m = (3.0 * mass_kg / (4.0 * std::f64::consts::PI * density)).cbrt();
        let temperature = disk.temperature_k(orbital_m);
        let composition = available_materials(temperature)
            .into_iter()
            .map(|material| Material {
                name: format!("{:?}", material.material),
                surface_material: material.material,
                abundance: None,
            })
            .collect();
        let speed = (constants::GRAVITATIONAL_CONSTANT * stellar_mass_kg / orbital_m).sqrt();
        bodies.push(Planetesimal {
            id: i as u64 + 1,
            mass_kg,
            radius_m,
            orbital_distance_au: orbital_au,
            velocity_m_s: Vector3::new(0.0, speed, 0.0),
            composition,
            formation_region: classify_region(
                orbital_au,
                bounds.habitable_inner_au,
                bounds.habitable_outer_au,
                bounds.water_frost_au,
            ),
        });
    }

    bodies
}
