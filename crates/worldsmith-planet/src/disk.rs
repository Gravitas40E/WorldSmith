//! Protoplanetary disk model and region boundaries.

use serde::{Deserialize, Serialize};
use worldsmith_math::constants;
use worldsmith_stellar::{frost_lines, habitable_zone};

use crate::{
    density::{midplane_pressure_pa, surface_density_kg_m2},
    temperature::disk_temperature_k,
    validation,
};

/// Protoplanetary disk parameters derived from stellar inputs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProtoplanetaryDisk {
    /// Parent stellar mass in solar masses.
    pub stellar_mass_solar: f64,
    /// Parent stellar luminosity in solar luminosities.
    pub stellar_luminosity_solar: f64,
    /// Stellar metallicity mass fraction.
    pub metallicity: f64,
    /// Disk age in megayears.
    pub age_myr: f64,
    /// Total disk mass in kilograms.
    pub disk_mass_kg: f64,
    /// Outer disk radius in meters.
    pub disk_radius_m: f64,
    /// Gas mass fraction.
    pub gas_fraction: f64,
    /// Dust and solids mass fraction.
    pub dust_fraction: f64,
    /// Surface density power-law exponent.
    pub surface_density_exponent: f64,
}

/// Important disk region boundaries in AU.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DiskRegionBounds {
    /// Conservative habitable zone inner edge.
    pub habitable_inner_au: f64,
    /// Conservative habitable zone outer edge.
    pub habitable_outer_au: f64,
    /// Water frost line.
    pub water_frost_au: f64,
    /// Ammonia frost line.
    pub ammonia_frost_au: f64,
    /// Methane frost line.
    pub methane_frost_au: f64,
}

impl ProtoplanetaryDisk {
    /// Creates a disk using MMSN-inspired scaling from the parent star.
    pub fn from_star(
        stellar_mass_solar: f64,
        luminosity_solar: f64,
        metallicity: f64,
        age_myr: f64,
    ) -> Self {
        let metallicity_scale = (metallicity / 0.0134).clamp(0.1, 5.0);
        let disk_mass_kg = 0.01 * stellar_mass_solar.powf(1.2) * constants::SOLAR_MASS;
        let disk_radius_m =
            80.0 * stellar_mass_solar.sqrt().max(0.5) * constants::ASTRONOMICAL_UNIT;
        let dust_fraction = (0.01 * metallicity_scale).clamp(0.001, 0.10);
        Self {
            stellar_mass_solar,
            stellar_luminosity_solar: luminosity_solar,
            metallicity,
            age_myr,
            disk_mass_kg,
            disk_radius_m,
            gas_fraction: 1.0 - dust_fraction,
            dust_fraction,
            surface_density_exponent: 1.5,
        }
    }

    /// Validates disk mass and radius.
    pub fn validate(&self) -> crate::errors::PlanetFormationResult<()> {
        validation::validate_disk_mass_kg(self.disk_mass_kg)?;
        validation::validate_positive_radius_m(self.disk_radius_m)?;
        Ok(())
    }

    /// Returns surface density at orbital distance.
    pub fn surface_density_kg_m2(&self, orbital_distance_m: f64) -> f64 {
        surface_density_kg_m2(
            self.disk_mass_kg,
            self.disk_radius_m,
            orbital_distance_m,
            self.surface_density_exponent,
        )
    }

    /// Returns disk temperature at orbital distance.
    pub fn temperature_k(&self, orbital_distance_m: f64) -> f64 {
        disk_temperature_k(
            self.stellar_luminosity_solar,
            orbital_distance_m,
            self.age_myr,
        )
    }

    /// Returns pressure proxy at orbital distance.
    pub fn pressure_pa(&self, orbital_distance_m: f64) -> f64 {
        midplane_pressure_pa(
            self.surface_density_kg_m2(orbital_distance_m),
            self.temperature_k(orbital_distance_m),
            self.stellar_mass_solar * constants::SOLAR_MASS,
            orbital_distance_m,
        )
    }
}

/// Computes disk region boundaries from stellar luminosity.
pub fn disk_regions(luminosity_solar: f64) -> DiskRegionBounds {
    let hz = habitable_zone(luminosity_solar);
    let frost = frost_lines(luminosity_solar);
    DiskRegionBounds {
        habitable_inner_au: hz.conservative_inner_au,
        habitable_outer_au: hz.conservative_outer_au,
        water_frost_au: frost.water_au,
        ammonia_frost_au: frost.ammonia_au,
        methane_frost_au: frost.methane_au,
    }
}
