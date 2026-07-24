//! Planetary surface chemistry: deterministic bulk mineral and chemical reservoirs.
//!
//! This module implements a V1 deterministic planetary surface chemistry model.
//! It models bulk silicate weathering, carbonate formation, and sedimentation
//! at a planetary scale.
//!
//! ## Responsibilities
//! - Owns `silicate_mass_kg`, `carbonate_mass_kg`, `oxidized_material_mass_kg`,
//!   `reduced_material_mass_kg`, `dissolved_mineral_mass_kg`,
//!   `weathering_rate_kg_per_s`, `sedimentation_rate_kg_per_s`,
//!   `weathering_index`, `surface_reactivity`, and `mineral_availability`
//!   per ADR-011.
//! - Reads `ClimateState`, `HydrologyState`, `CarbonCycleState`, `CryosphereState`,
//!   and `Planet` properties after `worldsmith.evolution.cryosphere`.
//!
//! ## Simplifying assumptions
//! 1. **Bulk reservoirs**: a single silicate and carbonate reservoir.
//!    No mineral species, reaction networks, or detailed mineralogy.
//! 2. **Temperature and water driven**: weathering intensity is a deterministic
//!    function of surface temperature and liquid water availability.
//! 3. **CO2-dependent acidity**: higher atmospheric CO2 increases chemical
//!    weathering rates.
//! 4. **Ice suppression**: cryosphere cover reduces weathering.
//! 5. **No aqueous chemistry**: no pH, ion speciation, or ocean chemistry.
//! 6. **No atmospheric chemistry**: atmospheric composition is read only.
//!
//! ## Future extensions
//! - detailed mineralogy
//! - reaction networks
//! - aqueous chemistry
//! - climate feedback via CO2 drawdown
//!
//! ## Ownership
//!
//! - **Reads**: `ClimateState`, `HydrologyState`, `CarbonCycleState`,
//!   `CryosphereState`, `Planet` properties
//! - **Writes**: `silicate_mass_kg`, `carbonate_mass_kg`,
//!   `oxidized_material_mass_kg`, `reduced_material_mass_kg`,
//!   `dissolved_mineral_mass_kg`, `weathering_rate_kg_per_s`,
//!   `sedimentation_rate_kg_per_s`, `weathering_index`,
//!   `surface_reactivity`, `mineral_availability`
//! - **Never modifies**: `ClimateState`, `HydrologyState`, `CarbonCycleState`,
//!   `CryosphereState`, `AtmosphereState`, `BiosphereState`,
//!   `InteriorState`, `VolcanismState`, `PlateTectonicsState`, `climate`,
//!   `ocean`, `magnetic_field`, `habitability`

use serde::{Deserialize, Serialize};
use worldsmith_models::{
    CarbonCycleState, ClimateState, CryosphereState,
    HydrologyState, Planet, PlanetId, SurfaceChemistryState,
};
use worldsmith_state::{FieldKey, SimulationEvent};
use worldsmith_traits::{ContractResult, ModuleContext, SimulationModule, StateWriter};

const DEFAULT_WEATHERING_CONSTANT: f64 = 1.0e9;
const DEFAULT_CARBONATE_PRECIPITATION_RATE: f64 = 1.0e8;
const DEFAULT_OXIDATION_RATE: f64 = 1.0e7;
const DEFAULT_SEDIMENTATION_RATE: f64 = 1.0e8;
const DEFAULT_DISSOLUTION_RATE: f64 = 1.0e9;
const DEFAULT_SILICATE_MASS_KG: f64 = 2.0e22;
const DEFAULT_CARBONATE_MASS_KG: f64 = 5.0e20;
const DEFAULT_OXIDIZED_MATERIAL_MASS_KG: f64 = 1.0e21;
const DEFAULT_REDUCED_MATERIAL_MASS_KG: f64 = 1.0e20;
const DEFAULT_DISSOLVED_MINERAL_MASS_KG: f64 = 1.0e18;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SurfaceChemistryConfig {
    pub weathering_constant: f64,
    pub carbonate_precipitation_rate: f64,
    pub oxidation_rate: f64,
    pub sedimentation_rate: f64,
    pub dissolution_rate: f64,
}

impl Default for SurfaceChemistryConfig {
    fn default() -> Self {
        Self {
            weathering_constant: DEFAULT_WEATHERING_CONSTANT,
            carbonate_precipitation_rate: DEFAULT_CARBONATE_PRECIPITATION_RATE,
            oxidation_rate: DEFAULT_OXIDATION_RATE,
            sedimentation_rate: DEFAULT_SEDIMENTATION_RATE,
            dissolution_rate: DEFAULT_DISSOLUTION_RATE,
        }
    }
}

pub struct SurfaceChemistryModule {
    config: SurfaceChemistryConfig,
    initialized: bool,
}

impl SurfaceChemistryModule {
    pub fn new(config: SurfaceChemistryConfig) -> Self {
        Self {
            config,
            initialized: false,
        }
    }

    fn initialize_reservoirs(&self, planet: &Planet) -> SurfaceChemistryState {
        let mut state = SurfaceChemistryState::default();
        state.silicate_mass_kg = DEFAULT_SILICATE_MASS_KG;
        state.carbonate_mass_kg = DEFAULT_CARBONATE_MASS_KG;
        state.oxidized_material_mass_kg = DEFAULT_OXIDIZED_MATERIAL_MASS_KG;
        state.reduced_material_mass_kg = DEFAULT_REDUCED_MATERIAL_MASS_KG;
        state.dissolved_mineral_mass_kg = DEFAULT_DISSOLVED_MINERAL_MASS_KG;
        if let Some(existing) = &planet.surface_chemistry_state {
            state.silicate_mass_kg = existing.silicate_mass_kg;
            state.carbonate_mass_kg = existing.carbonate_mass_kg;
            state.oxidized_material_mass_kg = existing.oxidized_material_mass_kg;
            state.reduced_material_mass_kg = existing.reduced_material_mass_kg;
            state.dissolved_mineral_mass_kg = existing.dissolved_mineral_mass_kg;
            state.weathering_rate_kg_per_s = existing.weathering_rate_kg_per_s;
            state.sedimentation_rate_kg_per_s = existing.sedimentation_rate_kg_per_s;
            state.weathering_index = existing.weathering_index;
            state.surface_reactivity = existing.surface_reactivity;
            state.mineral_availability = existing.mineral_availability;
        }
        state
    }

    fn compute_surface_chemistry(
        &self,
        chem: &mut SurfaceChemistryState,
        climate: &ClimateState,
        hydro: &HydrologyState,
        carbon: &CarbonCycleState,
        cryo: &CryosphereState,
        _planet: &Planet,
    ) {
        let surface_temp_k =
            climate.equilibrium_temperature_k + climate.greenhouse_temperature_offset_k;
        let water_factor = hydro.liquid_water_fraction;
        let cryo_factor = 1.0 - cryo.planetary_ice_fraction;

        let co2_factor = (carbon.atmospheric_co2_fraction * 1.0e4).clamp(0.1, 10.0);

        let temp_suitability = ((surface_temp_k - 250.0) / 50.0).clamp(0.0, 1.0);
        let weathering_intensity = self.config.weathering_constant
            * temp_suitability
            * water_factor
            * cryo_factor
            * co2_factor;

        let dissolved_input = weathering_intensity * 1.0;
        let dissolution_output = self.config.dissolution_rate * water_factor;

        let mut silicate = chem.silicate_mass_kg;
        let mut carbonate = chem.carbonate_mass_kg;
        let mut oxidized = chem.oxidized_material_mass_kg;
        let mut reduced = chem.reduced_material_mass_kg;
        let mut dissolved = chem.dissolved_mineral_mass_kg;

        silicate -= weathering_intensity * 1.0;
        silicate = silicate.max(0.0);

        dissolved += dissolved_input;
        dissolved -= dissolution_output;
        dissolved = dissolved.max(0.0);

        let carbonate_formation =
            dissolution_output * self.config.carbonate_precipitation_rate * 1.0;
        carbonate += carbonate_formation;

        let oxidation = self.config.oxidation_rate * temp_suitability * water_factor;
        reduced -= oxidation;
        reduced = reduced.max(0.0);
        oxidized += oxidation;

        let sedimentation = self.config.sedimentation_rate * dissolved.min(1.0);
        dissolved -= sedimentation;
        dissolved = dissolved.max(0.0);
        carbonate += sedimentation * 0.5;

        let weathering_index =
            (weathering_intensity / (self.config.weathering_constant + 1e-9)).clamp(0.0, 1.0);
        let surface_reactivity = (temp_suitability * water_factor).clamp(0.0, 1.0);
        let mineral_availability = (silicate / (DEFAULT_SILICATE_MASS_KG + 1e-9)).clamp(0.0, 1.0);

        chem.silicate_mass_kg = silicate;
        chem.carbonate_mass_kg = carbonate;
        chem.oxidized_material_mass_kg = oxidized;
        chem.reduced_material_mass_kg = reduced;
        chem.dissolved_mineral_mass_kg = dissolved;
        chem.weathering_rate_kg_per_s = weathering_intensity;
        chem.sedimentation_rate_kg_per_s = sedimentation;
        chem.weathering_index = weathering_index;
        chem.surface_reactivity = surface_reactivity;
        chem.mineral_availability = mineral_availability;
    }

    fn tick(
        &self,
        climate: &ClimateState,
        hydro: &HydrologyState,
        carbon: &CarbonCycleState,
        cryo: &CryosphereState,
        planet: &Planet,
    ) -> SurfaceChemistryState {
        let mut chem = SurfaceChemistryState::default();
        self.compute_surface_chemistry(&mut chem, climate, hydro, carbon, cryo, planet);
        chem
    }
}

impl Default for SurfaceChemistryModule {
    fn default() -> Self {
        Self::new(SurfaceChemistryConfig::default())
    }
}

impl SimulationModule for SurfaceChemistryModule {
    fn id(&self) -> &'static str {
        "worldsmith.evolution.surface_chemistry"
    }

    fn initialize(&mut self, state: &mut dyn StateWriter) -> ContractResult<()> {
        let planet_ids: Vec<_> = state.world().planets.keys().cloned().collect();
        for planet_id in planet_ids {
            if let Some(planet) = state.world().planets.get(&planet_id).cloned() {
                let chem = self.initialize_reservoirs(&planet);
                let mut updated = planet;
                updated.surface_chemistry_state = Some(chem);
                state.world_mut().planets.insert(updated.id, updated);
            }
        }
        self.initialized = true;
        Ok(())
    }

    fn update(
        &mut self,
        _context: ModuleContext,
        state: &mut dyn StateWriter,
    ) -> ContractResult<()> {
        if !self.initialized {
            return Ok(());
        }

        let snapshot: Vec<(PlanetId, Planet, Option<SurfaceChemistryState>)> = state
            .world()
            .planets
            .values()
            .map(|planet| {
                (
                    planet.id,
                    planet.clone(),
                    planet.surface_chemistry_state.clone(),
                )
            })
            .collect();

        for (_planet_id, planet, chem) in snapshot {
            let chem = match chem {
                Some(chem) => chem,
                None => continue,
            };

            let climate = match &planet.climate_state {
                Some(c) => c,
                None => continue,
            };
            let hydro = match &planet.hydrology_state {
                Some(h) => h,
                None => continue,
            };
            let carbon = match &planet.carbon_cycle_state {
                Some(c) => c,
                None => continue,
            };
            let cryo = match &planet.cryosphere_state {
                Some(c) => c,
                None => continue,
            };

            let updated = self.tick(climate, hydro, carbon, cryo, &planet);
            let mut updated_planet = planet;
            updated_planet.surface_chemistry_state = Some(updated);
            state
                .world_mut()
                .planets
                .insert(updated_planet.id, updated_planet);
        }

        Ok(())
    }

    fn reads(&self) -> Vec<FieldKey> {
        vec![
            FieldKey::SilicateMass,
            FieldKey::CarbonateMass,
            FieldKey::OxidizedMaterialMass,
            FieldKey::ReducedMaterialMass,
            FieldKey::DissolvedMineralMass,
            FieldKey::WeatheringRate,
            FieldKey::SedimentationRate,
            FieldKey::WeatheringIndex,
            FieldKey::SurfaceReactivity,
            FieldKey::MineralAvailability,
        ]
    }

    fn writes(&self) -> Vec<FieldKey> {
        vec![
            FieldKey::SilicateMass,
            FieldKey::CarbonateMass,
            FieldKey::OxidizedMaterialMass,
            FieldKey::ReducedMaterialMass,
            FieldKey::DissolvedMineralMass,
            FieldKey::WeatheringRate,
            FieldKey::SedimentationRate,
            FieldKey::WeatheringIndex,
            FieldKey::SurfaceReactivity,
            FieldKey::MineralAvailability,
        ]
    }

    fn shutdown(&mut self, _state: &mut dyn StateWriter) -> ContractResult<()> {
        Ok(())
    }

    fn publish_events(&mut self) -> Vec<SimulationEvent> {
        Vec::new()
    }

    fn consume_events(&mut self, _events: &[SimulationEvent]) -> ContractResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AtmosphereModule, BiosphereModule, CarbonCycleModule, ClimateModule, CoreEvolutionModule,
        CryosphereModule, HydrologyModule, MantleEvolutionModule, PlateTectonicsModule,
        VolcanismModule,
    };
    use worldsmith_engine::EngineBuilder;
    use worldsmith_math::Vector3;
    use worldsmith_models::{
        AtmosphericGas, AtmosphericProperties, BiosphereState, CarbonCycleState, CryosphereState,
        MeasuredValue, OceanProperties, OrbitalProperties, PhysicalProperties, Star, StarId,
        SurfaceChemistryState, SystemId,
    };

    fn earth_like_planet(planet_id: PlanetId, star_id: StarId) -> worldsmith_models::Planet {
        worldsmith_models::Planet {
            id: planet_id,
            name: format!("Planet {}", planet_id.0),
            class: worldsmith_models::PlanetClass::Terrestrial,
            planet_type: worldsmith_models::PlanetType::Rocky,
            system_id: SystemId(1),
            physical: PhysicalProperties {
                mass_kg: MeasuredValue {
                    value: 5.972e24,
                    unit: "kg".into(),
                    provenance: None,
                },
                radius_m: MeasuredValue {
                    value: 6.371e6,
                    unit: "m".into(),
                    provenance: None,
                },
                density_kg_m3: None,
                surface_gravity_m_s2: None,
            },
            orbit: OrbitalProperties {
                parent: worldsmith_models::BodyReference::Star(star_id),
                semi_major_axis_m: MeasuredValue {
                    value: 1.496e11,
                    unit: "m".into(),
                    provenance: None,
                },
                semi_minor_axis_m: None,
                eccentricity: MeasuredValue {
                    value: 0.0167,
                    unit: "dimensionless".into(),
                    provenance: None,
                },
                inclination_rad: MeasuredValue {
                    value: 0.0,
                    unit: "rad".into(),
                    provenance: None,
                },
                orbital_period_s: None,
                rotation_period_s: None,
                axial_tilt_rad: None,
            },
            geology: None,
            atmosphere: Some(AtmosphericProperties {
                atmosphere_type: worldsmith_models::AtmosphereType::None,
                pressure_pa: None,
                density_kg_m3: None,
                scale_height_m: None,
                layers: vec![],
                composition: vec![],
                cloud_coverage: None,
                greenhouse_gases: vec![],
            }),
            atmosphere_state: Some(AtmosphereState {
                atmospheric_mass_kg: 5.15e18,
                surface_pressure_pa: 101_325.0,
                mean_temperature_k: 288.0,
                atmosphere_composition: vec![AtmosphericGas {
                    molecule: worldsmith_models::Molecule {
                        formula: "CO2".into(),
                        name: "Carbon Dioxide".into(),
                        molar_mass_kg_mol: Some(MeasuredValue {
                            value: 0.04401,
                            unit: "kg/mol".into(),
                            provenance: None,
                        }),
                    },
                    abundance: MeasuredValue {
                        value: 0.0004,
                        unit: "dimensionless".into(),
                        provenance: None,
                    },
                    is_greenhouse: true,
                }],
            }),
            hydrology_state: Some(HydrologyState {
                total_water_mass_kg: 1.4e21,
                ocean_mass_kg: 1.4e21,
                atmospheric_water_mass_kg: 1.0e16,
                ice_mass_kg: 0.0,
                liquid_water_fraction: 1.0,
            }),
            climate_state: Some(ClimateState {
                equilibrium_temperature_k: 255.0,
                greenhouse_temperature_offset_k: 33.0,
                planetary_albedo: 0.3,
                climate_classification: worldsmith_models::ClimateType::Temperate,
            }),
            carbon_cycle_state: Some(CarbonCycleState::default()),
            biosphere_state: Some(BiosphereState::default()),
            habitability_state: Some(HabitabilityState::default()),
            classification_state: Some(PlanetClassificationState::default()),
            surface_chemistry_state: Some(SurfaceChemistryState::default()),
            cryosphere_state: Some(CryosphereState::default()),
            interior: None,
            volcanism: Some(worldsmith_models::VolcanismState {
                volcanic_flux: 1.0e10,
                volcanic_activity: worldsmith_models::VolcanicActivity::Moderate,
                magma_generation_rate: 1.0e10,
            }),
            plate_tectonics: Some(worldsmith_models::PlateTectonicsState {
                plate_velocity: 5.0,
                crustal_recycling_rate: 1.0e9,
                tectonic_activity: worldsmith_models::TectonicActivity::Moderate,
            }),
            climate: None,
            ocean: Some(OceanProperties {
                ocean_type: worldsmith_models::OceanType::Water,
                coverage: None,
                average_depth_m: None,
                composition: vec![],
            }),
            magnetic_field: None,
            habitability: None,
            position_m: Vector3::ZERO,
            velocity_m_s: Vector3::ZERO,
            moons: Vec::new(),
        }
    }

    fn default_star() -> Star {
        Star {
            id: StarId(1),
            name: "Sol".into(),
            spectral_type: worldsmith_models::SpectralType::G,
            class: worldsmith_models::StarClass::MainSequence,
            mass_kg: MeasuredValue {
                value: 1.989e30,
                unit: "kg".into(),
                provenance: None,
            },
            radius_m: MeasuredValue {
                value: 6.96e8,
                unit: "m".into(),
                provenance: None,
            },
            luminosity_w: MeasuredValue {
                value: 5.0e26,
                unit: "W".into(),
                provenance: None,
            },
            effective_temperature_k: MeasuredValue {
                value: 5778.0,
                unit: "K".into(),
                provenance: None,
            },
            surface_gravity_m_s2: MeasuredValue {
                value: 274.0,
                unit: "m/s2".into(),
                provenance: None,
            },
            metallicity: MeasuredValue {
                value: 0.0,
                unit: "dimensionless".into(),
                provenance: None,
            },
            rotation_period_s: None,
            age_s: Some(MeasuredValue {
                value: 4.6e17,
                unit: "s".into(),
                provenance: None,
            }),
            position_m: Vector3::ZERO,
            velocity_m_s: Vector3::ZERO,
        }
    }

    #[test]
    fn module_constructs_with_defaults() {
        let module = SurfaceChemistryModule::default();
        assert_eq!(module.id(), "worldsmith.evolution.surface_chemistry");
    }

    #[test]
    fn surface_chemistry_mutates_surface_chemistry_state() {
        let mut engine = EngineBuilder::new()
            .with_seed(7)
            .register_module(Box::new(CoreEvolutionModule::default()))
            .register_module(Box::new(MantleEvolutionModule::default()))
            .register_module(Box::new(VolcanismModule::default()))
            .register_module(Box::new(PlateTectonicsModule::default()))
            .register_module(Box::new(AtmosphereModule::default()))
            .register_module(Box::new(HydrologyModule::default()))
            .register_module(Box::new(ClimateModule::default()))
            .register_module(Box::new(CarbonCycleModule::default()))
            .register_module(Box::new(BiosphereModule::default()))
            .register_module(Box::new(CryosphereModule::default()))
            .register_module(Box::new(SurfaceChemistryModule::default()))
            .build()
            .unwrap();

        let planet_id = PlanetId(1);
        let star_id = StarId(1);
        let planet = earth_like_planet(planet_id, star_id);

        engine.state_mut().planets.insert(planet_id, planet);
        engine.state_mut().stars.insert(star_id, default_star());
        let _ = engine.initialize();
        let _ = engine.tick(100.0);
        let _ = engine.tick(100.0);

        let updated = engine
            .state()
            .planets
            .get(&planet_id)
            .and_then(|p| p.surface_chemistry_state.clone())
            .unwrap();

        assert!(
            updated.silicate_mass_kg.is_finite(),
            "silicate mass must be finite"
        );
        assert!(
            updated.carbonate_mass_kg.is_finite(),
            "carbonate mass must be finite"
        );
        assert!(
            updated.weathering_rate_kg_per_s >= 0.0,
            "weathering rate must be non-negative"
        );
        assert!(
            updated.sedimentation_rate_kg_per_s >= 0.0,
            "sedimentation rate must be non-negative"
        );
        assert!(
            updated.weathering_index >= 0.0 && updated.weathering_index <= 1.0,
            "weathering index must be in [0, 1]"
        );
        assert!(
            updated.surface_reactivity >= 0.0 && updated.surface_reactivity <= 1.0,
            "surface reactivity must be in [0, 1]"
        );
        assert!(
            updated.mineral_availability >= 0.0 && updated.mineral_availability <= 1.0,
            "mineral availability must be in [0, 1]"
        );
    }

    #[test]
    fn climate_drives_surface_chemistry_state() {
        let mut engine = EngineBuilder::new()
            .with_seed(7)
            .register_module(Box::new(SurfaceChemistryModule::default()))
            .build()
            .unwrap();

        let planet_id = PlanetId(1);
        let star_id = StarId(1);
        let mut planet = earth_like_planet(planet_id, star_id);
        planet.climate_state = Some(ClimateState {
            equilibrium_temperature_k: 290.0,
            greenhouse_temperature_offset_k: 0.0,
            planetary_albedo: 0.3,
            climate_classification: worldsmith_models::ClimateType::Temperate,
        });

        engine.state_mut().planets.insert(planet_id, planet);
        engine.state_mut().stars.insert(star_id, default_star());
        let _ = engine.initialize();
        let _ = engine.tick(100.0);

        let updated = engine
            .state()
            .planets
            .get(&planet_id)
            .and_then(|p| p.surface_chemistry_state.clone())
            .unwrap();

        assert!(
            updated.weathering_rate_kg_per_s > 0.0,
            "warmer climate should produce positive weathering"
        );
    }

    #[test]
    fn snapshots_contain_surface_chemistry_state() {
        let mut engine = EngineBuilder::new()
            .with_seed(7)
            .register_module(Box::new(CoreEvolutionModule::default()))
            .register_module(Box::new(MantleEvolutionModule::default()))
            .register_module(Box::new(VolcanismModule::default()))
            .register_module(Box::new(PlateTectonicsModule::default()))
            .register_module(Box::new(AtmosphereModule::default()))
            .register_module(Box::new(HydrologyModule::default()))
            .register_module(Box::new(ClimateModule::default()))
            .register_module(Box::new(CarbonCycleModule::default()))
            .register_module(Box::new(BiosphereModule::default()))
            .register_module(Box::new(CryosphereModule::default()))
            .register_module(Box::new(SurfaceChemistryModule::default()))
            .build()
            .unwrap();

        let planet_id = PlanetId(1);
        let star_id = StarId(1);
        let planet = earth_like_planet(planet_id, star_id);

        engine.state_mut().planets.insert(planet_id, planet);
        engine.state_mut().stars.insert(star_id, default_star());
        let _ = engine.initialize();
        let _ = engine.tick(100.0);

        let snapshot = engine.latest_snapshot().expect("snapshot");
        let planet_snapshot = snapshot
            .planets
            .iter()
            .find(|p| p.id == planet_id)
            .expect("planet snapshot must exist");

        assert!(
            planet_snapshot.planet.surface_chemistry_state.is_some(),
            "snapshot must preserve surface chemistry state"
        );
    }

    #[test]
    fn surface_chemistry_does_not_modify_carbon_cycle_state() {
        let mut engine = EngineBuilder::new()
            .with_seed(7)
            .register_module(Box::new(SurfaceChemistryModule::default()))
            .build()
            .unwrap();

        let planet_id = PlanetId(1);
        let star_id = StarId(1);
        let planet = earth_like_planet(planet_id, star_id);
        let initial_carbon = planet.carbon_cycle_state.clone().unwrap();

        engine.state_mut().planets.insert(planet_id, planet);
        engine.state_mut().stars.insert(star_id, default_star());
        let _ = engine.initialize();
        let _ = engine.tick(100.0);

        let updated = engine
            .state()
            .planets
            .get(&planet_id)
            .and_then(|p| p.carbon_cycle_state.clone())
            .unwrap();

        assert_eq!(
            updated.atmospheric_carbon_mass_kg, initial_carbon.atmospheric_carbon_mass_kg,
            "SurfaceChemistryModule must not mutate CarbonCycleState::atmospheric_carbon_mass_kg"
        );
        assert_eq!(
            updated.weathering_flux_kg_per_s, initial_carbon.weathering_flux_kg_per_s,
            "SurfaceChemistryModule must not mutate CarbonCycleState::weathering_flux_kg_per_s"
        );
    }
}
