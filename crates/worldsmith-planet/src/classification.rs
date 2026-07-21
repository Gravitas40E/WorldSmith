//! Planet classification from mass, density, composition, and formation context.

use worldsmith_math::constants;
use worldsmith_models::{PlanetClass, PlanetType};

use crate::embryo::{bulk_density_kg_m3, PlanetaryEmbryo};

/// Classifies an embryo into model-layer planet class and type.
pub fn classify_embryo(embryo: &PlanetaryEmbryo) -> (PlanetClass, PlanetType) {
    let earth_masses = embryo.mass_kg / constants::EARTH_MASS;
    let density = bulk_density_kg_m3(embryo.composition);
    let volatile_fraction = embryo.composition.water_fraction + embryo.composition.ice_fraction;
    let gas_fraction = embryo.composition.gas_fraction;

    let class = if earth_masses < 0.08 {
        PlanetClass::Dwarf
    } else if gas_fraction > 0.15 || earth_masses > 80.0 {
        PlanetClass::GasGiant
    } else if volatile_fraction > 0.30 && earth_masses > 10.0 {
        PlanetClass::IceGiant
    } else if earth_masses > 2.0 && earth_masses <= 10.0 && volatile_fraction < 0.25 {
        PlanetClass::SuperEarth
    } else if earth_masses > 2.0 {
        PlanetClass::MiniNeptune
    } else {
        PlanetClass::Terrestrial
    };

    let planet_type = if gas_fraction > 0.15 {
        PlanetType::Gas
    } else if volatile_fraction > 0.35 {
        PlanetType::Ice
    } else if embryo.composition.water_fraction > 0.20 {
        PlanetType::Ocean
    } else if density > 2_500.0 {
        PlanetType::Rocky
    } else {
        PlanetType::Ice
    };

    (class, planet_type)
}
