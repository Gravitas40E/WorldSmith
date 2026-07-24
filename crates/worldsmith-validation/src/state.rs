//! State validation helpers.
//!
//! Verify that planetary state does not contain NaN, Inf, or structurally
//! invalid values.  These checks operate on `Planet` structs extracted from
//! `WorldState`.

use worldsmith_models::Planet;
use worldsmith_state::WorldState;

/// Errors detected during state validation.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum StateValidationError {
    /// A numeric field contains NaN.
    #[error("planet {planet_id} field {field} is NaN")]
    Nan {
        /// Planet identifier.
        planet_id: String,
        /// Field name.
        field: &'static str,
    },
    /// A numeric field contains infinity.
    #[error("planet {planet_id} field {field} is infinite")]
    Infinity {
        /// Planet identifier.
        planet_id: String,
        /// Field name.
        field: &'static str,
    },
    /// An `Option` state is `None` when it should be present.
    #[error("planet {planet_id} missing state {state}")]
    MissingState {
        /// Planet identifier.
        planet_id: String,
        /// State name.
        state: &'static str,
    },
    /// A categorical enum value is invalid.
    #[error("planet {planet_id} unexpected enum discriminant")]
    InvalidEnum {
        /// Planet identifier.
        planet_id: String,
    },
}

/// Validates all planets in `WorldState` for structural correctness.
///
/// Checks:
/// - No NaN/Inf in interior, volcanism, plate_tectonics, atmosphere, or
///   hydrology floats.
/// - Option state consistency.
/// - Atmospheric composition fractions in [0, 1] and sum approximately 1.
/// - Hydrology reservoir bounds and conservation.
pub fn validate_state(state: &WorldState) -> Result<(), StateValidationError> {
    for (id, planet) in state.planets.iter() {
        let planet_id = format!("{id:?}");
        validate_planet_state(planet, &planet_id)?;
    }
    Ok(())
}

fn validate_planet_state(planet: &Planet, planet_id: &str) -> Result<(), StateValidationError> {
    if let Some(interior) = &planet.interior {
        if interior.core_temperature.is_nan() {
            return Err(StateValidationError::Nan {
                planet_id: planet_id.to_string(),
                field: "core_temperature",
            });
        }
        if interior.core_temperature.is_infinite() {
            return Err(StateValidationError::Infinity {
                planet_id: planet_id.to_string(),
                field: "core_temperature",
            });
        }
        if interior.mantle_temperature.is_nan() {
            return Err(StateValidationError::Nan {
                planet_id: planet_id.to_string(),
                field: "mantle_temperature",
            });
        }
        if interior.mantle_temperature.is_infinite() {
            return Err(StateValidationError::Infinity {
                planet_id: planet_id.to_string(),
                field: "mantle_temperature",
            });
        }
        if interior.heat_flux.is_nan() {
            return Err(StateValidationError::Nan {
                planet_id: planet_id.to_string(),
                field: "heat_flux",
            });
        }
        if interior.heat_flux.is_infinite() {
            return Err(StateValidationError::Infinity {
                planet_id: planet_id.to_string(),
                field: "heat_flux",
            });
        }
        if interior.radiogenic_heat.is_nan() {
            return Err(StateValidationError::Nan {
                planet_id: planet_id.to_string(),
                field: "radiogenic_heat",
            });
        }
        if interior.radiogenic_heat.is_infinite() {
            return Err(StateValidationError::Infinity {
                planet_id: planet_id.to_string(),
                field: "radiogenic_heat",
            });
        }
    }

    if let Some(volcanism) = &planet.volcanism {
        if volcanism.volcanic_flux.is_nan() {
            return Err(StateValidationError::Nan {
                planet_id: planet_id.to_string(),
                field: "volcanic_flux",
            });
        }
        if volcanism.volcanic_flux.is_infinite() {
            return Err(StateValidationError::Infinity {
                planet_id: planet_id.to_string(),
                field: "volcanic_flux",
            });
        }
        if volcanism.magma_generation_rate.is_nan() {
            return Err(StateValidationError::Nan {
                planet_id: planet_id.to_string(),
                field: "magma_generation_rate",
            });
        }
        if volcanism.magma_generation_rate.is_infinite() {
            return Err(StateValidationError::Infinity {
                planet_id: planet_id.to_string(),
                field: "magma_generation_rate",
            });
        }
    }

    if let Some(plate_tectonics) = &planet.plate_tectonics {
        if plate_tectonics.plate_velocity.is_nan() {
            return Err(StateValidationError::Nan {
                planet_id: planet_id.to_string(),
                field: "plate_velocity",
            });
        }
        if plate_tectonics.plate_velocity.is_infinite() {
            return Err(StateValidationError::Infinity {
                planet_id: planet_id.to_string(),
                field: "plate_velocity",
            });
        }
        if plate_tectonics.crustal_recycling_rate.is_nan() {
            return Err(StateValidationError::Nan {
                planet_id: planet_id.to_string(),
                field: "crustal_recycling_rate",
            });
        }
        if plate_tectonics.crustal_recycling_rate.is_infinite() {
            return Err(StateValidationError::Infinity {
                planet_id: planet_id.to_string(),
                field: "crustal_recycling_rate",
            });
        }
    }

    if let Some(atmosphere) = &planet.atmosphere_state {
        if atmosphere.atmospheric_mass_kg.is_nan() {
            return Err(StateValidationError::Nan {
                planet_id: planet_id.to_string(),
                field: "atmospheric_mass_kg",
            });
        }
        if atmosphere.atmospheric_mass_kg.is_infinite() {
            return Err(StateValidationError::Infinity {
                planet_id: planet_id.to_string(),
                field: "atmospheric_mass_kg",
            });
        }
        if atmosphere.surface_pressure_pa.is_nan() {
            return Err(StateValidationError::Nan {
                planet_id: planet_id.to_string(),
                field: "surface_pressure_pa",
            });
        }
        if atmosphere.surface_pressure_pa.is_infinite() {
            return Err(StateValidationError::Infinity {
                planet_id: planet_id.to_string(),
                field: "surface_pressure_pa",
            });
        }
        if atmosphere.mean_temperature_k.is_nan() {
            return Err(StateValidationError::Nan {
                planet_id: planet_id.to_string(),
                field: "mean_temperature_k",
            });
        }
        if atmosphere.mean_temperature_k.is_infinite() {
            return Err(StateValidationError::Infinity {
                planet_id: planet_id.to_string(),
                field: "mean_temperature_k",
            });
        }

        let total: f64 = atmosphere
            .atmosphere_composition
            .iter()
            .fold(0.0, |s, g| s + g.abundance.value);
        for gas in &atmosphere.atmosphere_composition {
            let frac = gas.abundance.value;
            if frac < 0.0 || frac > 1.0 {
                return Err(StateValidationError::InvalidEnum {
                    planet_id: planet_id.to_string(),
                });
            }
        }
        if (total - 1.0).abs() > 0.01 {
            return Err(StateValidationError::InvalidEnum {
                planet_id: planet_id.to_string(),
            });
        }
    }

    if let Some(hydrology) = &planet.hydrology_state {
        if hydrology.total_water_mass_kg.is_nan() {
            return Err(StateValidationError::Nan {
                planet_id: planet_id.to_string(),
                field: "total_water_mass_kg",
            });
        }
        if hydrology.total_water_mass_kg.is_infinite() {
            return Err(StateValidationError::Infinity {
                planet_id: planet_id.to_string(),
                field: "total_water_mass_kg",
            });
        }
        if hydrology.ocean_mass_kg.is_nan() {
            return Err(StateValidationError::Nan {
                planet_id: planet_id.to_string(),
                field: "ocean_mass_kg",
            });
        }
        if hydrology.ocean_mass_kg.is_infinite() {
            return Err(StateValidationError::Infinity {
                planet_id: planet_id.to_string(),
                field: "ocean_mass_kg",
            });
        }
        if hydrology.atmospheric_water_mass_kg.is_nan() {
            return Err(StateValidationError::Nan {
                planet_id: planet_id.to_string(),
                field: "atmospheric_water_mass_kg",
            });
        }
        if hydrology.atmospheric_water_mass_kg.is_infinite() {
            return Err(StateValidationError::Infinity {
                planet_id: planet_id.to_string(),
                field: "atmospheric_water_mass_kg",
            });
        }
        if hydrology.ice_mass_kg.is_nan() {
            return Err(StateValidationError::Nan {
                planet_id: planet_id.to_string(),
                field: "ice_mass_kg",
            });
        }
        if hydrology.ice_mass_kg.is_infinite() {
            return Err(StateValidationError::Infinity {
                planet_id: planet_id.to_string(),
                field: "ice_mass_kg",
            });
        }
        if hydrology.liquid_water_fraction.is_nan() {
            return Err(StateValidationError::Nan {
                planet_id: planet_id.to_string(),
                field: "liquid_water_fraction",
            });
        }
        if hydrology.liquid_water_fraction.is_infinite() {
            return Err(StateValidationError::Infinity {
                planet_id: planet_id.to_string(),
                field: "liquid_water_fraction",
            });
        }
        if hydrology.liquid_water_fraction < 0.0 || hydrology.liquid_water_fraction > 1.0 {
            return Err(StateValidationError::InvalidEnum {
                planet_id: planet_id.to_string(),
            });
        }
        if hydrology.ocean_mass_kg < 0.0
            || hydrology.atmospheric_water_mass_kg < 0.0
            || hydrology.ice_mass_kg < 0.0
        {
            return Err(StateValidationError::InvalidEnum {
                planet_id: planet_id.to_string(),
            });
        }
        if hydrology.ocean_mass_kg + hydrology.atmospheric_water_mass_kg + hydrology.ice_mass_kg
            > hydrology.total_water_mass_kg + 1e-3 * hydrology.total_water_mass_kg.max(1.0)
        {
            return Err(StateValidationError::InvalidEnum {
                planet_id: planet_id.to_string(),
            });
        }
    }

    // Climate validation.
    if let Some(climate) = &planet.climate_state {
        if climate.equilibrium_temperature_k.is_nan()
            || climate.greenhouse_temperature_offset_k.is_nan()
            || climate.planetary_albedo.is_nan()
        {
            return Err(StateValidationError::Nan {
                planet_id: planet_id.to_string(),
                field: "climate_equilibrium_temperature_k",
            });
        }
        if !climate.equilibrium_temperature_k.is_finite()
            || !climate.greenhouse_temperature_offset_k.is_finite()
            || !climate.planetary_albedo.is_finite()
        {
            return Err(StateValidationError::Infinity {
                planet_id: planet_id.to_string(),
                field: "climate_temperature",
            });
        }
        if climate.planetary_albedo < 0.0 || climate.planetary_albedo > 1.0 {
            return Err(StateValidationError::InvalidEnum {
                planet_id: planet_id.to_string(),
            });
        }
        if climate.greenhouse_temperature_offset_k < 0.0 {
            return Err(StateValidationError::InvalidEnum {
                planet_id: planet_id.to_string(),
            });
        }
    }

    // Carbon cycle validation.
    if let Some(carbon) = &planet.carbon_cycle_state {
        if carbon.atmospheric_carbon_mass_kg.is_nan()
            || carbon.ocean_carbon_mass_kg.is_nan()
            || carbon.lithosphere_carbon_mass_kg.is_nan()
            || carbon.volcanic_carbon_flux_kg_per_s.is_nan()
            || carbon.weathering_flux_kg_per_s.is_nan()
            || carbon.ocean_exchange_flux_kg_per_s.is_nan()
        {
            return Err(StateValidationError::Nan {
                planet_id: planet_id.to_string(),
                field: "carbon_cycle_reservoir_or_flux",
            });
        }
        if !carbon.atmospheric_carbon_mass_kg.is_finite()
            || !carbon.ocean_carbon_mass_kg.is_finite()
            || !carbon.lithosphere_carbon_mass_kg.is_finite()
            || !carbon.volcanic_carbon_flux_kg_per_s.is_finite()
            || !carbon.weathering_flux_kg_per_s.is_finite()
            || !carbon.ocean_exchange_flux_kg_per_s.is_finite()
        {
            return Err(StateValidationError::Infinity {
                planet_id: planet_id.to_string(),
                field: "carbon_cycle_reservoir_or_flux",
            });
        }
        if carbon.atmospheric_carbon_mass_kg < 0.0
            || carbon.ocean_carbon_mass_kg < 0.0
            || carbon.lithosphere_carbon_mass_kg < 0.0
        {
            return Err(StateValidationError::InvalidEnum {
                planet_id: planet_id.to_string(),
            });
        }
    }

    // Biosphere validation.
    if let Some(bio) = &planet.biosphere_state {
        if bio.total_biomass_kg.is_nan()
            || bio.terrestrial_biomass_kg.is_nan()
            || bio.marine_biomass_kg.is_nan()
            || bio.dead_organic_carbon_kg.is_nan()
            || bio.productivity_rate_kg_per_s.is_nan()
            || bio.respiration_rate_kg_per_s.is_nan()
        {
            return Err(StateValidationError::Nan {
                planet_id: planet_id.to_string(),
                field: "biosphere_reservoir_or_flux",
            });
        }
        if !bio.total_biomass_kg.is_finite()
            || !bio.terrestrial_biomass_kg.is_finite()
            || !bio.marine_biomass_kg.is_finite()
            || !bio.dead_organic_carbon_kg.is_finite()
            || !bio.productivity_rate_kg_per_s.is_finite()
            || !bio.respiration_rate_kg_per_s.is_finite()
        {
            return Err(StateValidationError::Infinity {
                planet_id: planet_id.to_string(),
                field: "biosphere_reservoir_or_flux",
            });
        }
        if bio.total_biomass_kg < 0.0
            || bio.terrestrial_biomass_kg < 0.0
            || bio.marine_biomass_kg < 0.0
            || bio.dead_organic_carbon_kg < 0.0
        {
            return Err(StateValidationError::InvalidEnum {
                planet_id: planet_id.to_string(),
            });
        }
    }

    // Cryosphere validation.
    if let Some(cryo) = &planet.cryosphere_state {
        if cryo.continental_ice_mass_kg.is_nan()
            || cryo.sea_ice_mass_kg.is_nan()
            || cryo.snow_mass_kg.is_nan()
            || cryo.melt_rate_kg_per_s.is_nan()
            || cryo.freeze_rate_kg_per_s.is_nan()
            || cryo.planetary_ice_fraction.is_nan()
            || cryo.cryosphere_albedo_modifier.is_nan()
            || cryo.sea_level_offset_m.is_nan()
        {
            return Err(StateValidationError::Nan {
                planet_id: planet_id.to_string(),
                field: "cryosphere_reservoir_or_flux",
            });
        }
        if !cryo.continental_ice_mass_kg.is_finite()
            || !cryo.sea_ice_mass_kg.is_finite()
            || !cryo.snow_mass_kg.is_finite()
            || !cryo.melt_rate_kg_per_s.is_finite()
            || !cryo.freeze_rate_kg_per_s.is_finite()
        {
            return Err(StateValidationError::Infinity {
                planet_id: planet_id.to_string(),
                field: "cryosphere_reservoir_or_flux",
            });
        }
        if cryo.continental_ice_mass_kg < 0.0
            || cryo.sea_ice_mass_kg < 0.0
            || cryo.snow_mass_kg < 0.0
            || cryo.melt_rate_kg_per_s < 0.0
            || cryo.freeze_rate_kg_per_s < 0.0
        {
            return Err(StateValidationError::InvalidEnum {
                planet_id: planet_id.to_string(),
            });
        }
        if cryo.permanent_ice_fraction < 0.0 || cryo.permanent_ice_fraction > 1.0 {
            return Err(StateValidationError::InvalidEnum {
                planet_id: planet_id.to_string(),
            });
        }
        if cryo.seasonal_snow_fraction < 0.0 || cryo.seasonal_snow_fraction > 1.0 {
            return Err(StateValidationError::InvalidEnum {
                planet_id: planet_id.to_string(),
            });
        }
        if cryo.planetary_ice_fraction < 0.0 || cryo.planetary_ice_fraction > 1.0 {
            return Err(StateValidationError::InvalidEnum {
                planet_id: planet_id.to_string(),
            });
        }
    }

    // Surface chemistry validation.
    if let Some(chem) = &planet.surface_chemistry_state {
        if chem.silicate_mass_kg.is_nan()
            || chem.carbonate_mass_kg.is_nan()
            || chem.oxidized_material_mass_kg.is_nan()
            || chem.reduced_material_mass_kg.is_nan()
            || chem.dissolved_mineral_mass_kg.is_nan()
            || chem.weathering_rate_kg_per_s.is_nan()
            || chem.sedimentation_rate_kg_per_s.is_nan()
            || chem.weathering_index.is_nan()
            || chem.surface_reactivity.is_nan()
            || chem.mineral_availability.is_nan()
        {
            return Err(StateValidationError::Nan {
                planet_id: planet_id.to_string(),
                field: "surface_chemistry_reservoir_or_flux",
            });
        }
        if !chem.silicate_mass_kg.is_finite()
            || !chem.carbonate_mass_kg.is_finite()
            || !chem.oxidized_material_mass_kg.is_finite()
            || !chem.reduced_material_mass_kg.is_finite()
            || !chem.dissolved_mineral_mass_kg.is_finite()
            || !chem.weathering_rate_kg_per_s.is_finite()
            || !chem.sedimentation_rate_kg_per_s.is_finite()
        {
            return Err(StateValidationError::Infinity {
                planet_id: planet_id.to_string(),
                field: "surface_chemistry_reservoir_or_flux",
            });
        }
        if chem.silicate_mass_kg < 0.0
            || chem.carbonate_mass_kg < 0.0
            || chem.oxidized_material_mass_kg < 0.0
            || chem.reduced_material_mass_kg < 0.0
            || chem.dissolved_mineral_mass_kg < 0.0
            || chem.weathering_rate_kg_per_s < 0.0
            || chem.sedimentation_rate_kg_per_s < 0.0
        {
            return Err(StateValidationError::InvalidEnum {
                planet_id: planet_id.to_string(),
            });
        }
        if chem.weathering_index < 0.0 || chem.weathering_index > 1.0 {
            return Err(StateValidationError::InvalidEnum {
                planet_id: planet_id.to_string(),
            });
        }
        if chem.surface_reactivity < 0.0 || chem.surface_reactivity > 1.0 {
            return Err(StateValidationError::InvalidEnum {
                planet_id: planet_id.to_string(),
            });
        }
        if chem.mineral_availability < 0.0 || chem.mineral_availability > 1.0 {
            return Err(StateValidationError::InvalidEnum {
                planet_id: planet_id.to_string(),
            });
        }
    }

    // Habitability validation.
    if let Some(hab) = &planet.habitability_state {
        if hab.overall_habitability_index.is_nan()
            || hab.surface_habitability_index.is_nan()
            || hab.ocean_habitability_index.is_nan()
            || hab.biological_potential_index.is_nan()
            || hab.climate_stability_index.is_nan()
            || hab.water_availability_index.is_nan()
            || hab.atmosphere_suitability_index.is_nan()
        {
            return Err(StateValidationError::Nan {
                planet_id: planet_id.to_string(),
                field: "habitability_index",
            });
        }
        if !hab.overall_habitability_index.is_finite()
            || !hab.surface_habitability_index.is_finite()
            || !hab.ocean_habitability_index.is_finite()
            || !hab.biological_potential_index.is_finite()
            || !hab.climate_stability_index.is_finite()
            || !hab.water_availability_index.is_finite()
            || !hab.atmosphere_suitability_index.is_finite()
        {
            return Err(StateValidationError::Infinity {
                planet_id: planet_id.to_string(),
                field: "habitability_index",
            });
        }
        if hab.overall_habitability_index < 0.0 || hab.overall_habitability_index > 1.0 {
            return Err(StateValidationError::InvalidEnum {
                planet_id: planet_id.to_string(),
            });
        }
        if hab.surface_habitability_index < 0.0 || hab.surface_habitability_index > 1.0 {
            return Err(StateValidationError::InvalidEnum {
                planet_id: planet_id.to_string(),
            });
        }
        if hab.ocean_habitability_index < 0.0 || hab.ocean_habitability_index > 1.0 {
            return Err(StateValidationError::InvalidEnum {
                planet_id: planet_id.to_string(),
            });
        }
        if hab.biological_potential_index < 0.0 || hab.biological_potential_index > 1.0 {
            return Err(StateValidationError::InvalidEnum {
                planet_id: planet_id.to_string(),
            });
        }
        if hab.climate_stability_index < 0.0 || hab.climate_stability_index > 1.0 {
            return Err(StateValidationError::InvalidEnum {
                planet_id: planet_id.to_string(),
            });
        }
        if hab.water_availability_index < 0.0 || hab.water_availability_index > 1.0 {
            return Err(StateValidationError::InvalidEnum {
                planet_id: planet_id.to_string(),
            });
        }
        if hab.atmosphere_suitability_index < 0.0 || hab.atmosphere_suitability_index > 1.0 {
            return Err(StateValidationError::InvalidEnum {
                planet_id: planet_id.to_string(),
            });
        }
    }

    // Planet classification validation.
    if let Some(cls) = &planet.classification_state {
        if cls.confidence_score.is_nan() {
            return Err(StateValidationError::Nan {
                planet_id: planet_id.to_string(),
                field: "classification_confidence",
            });
        }
        if !cls.confidence_score.is_finite() {
            return Err(StateValidationError::Infinity {
                planet_id: planet_id.to_string(),
                field: "classification_confidence",
            });
        }
        if cls.confidence_score < 0.0 || cls.confidence_score > 1.0 {
            return Err(StateValidationError::InvalidEnum {
                planet_id: planet_id.to_string(),
            });
        }
        if cls.classification_summary.is_empty() {
            return Err(StateValidationError::InvalidEnum {
                planet_id: planet_id.to_string(),
            });
        }
    }

    Ok(())
}
