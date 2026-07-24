//! Planetary habitability assessment: deterministic evaluation of simulation outputs.
//!
//! This module implements a V1 deterministic planetary habitability assessment.
//! It reads outputs from all physical evolution modules and produces a single
//! `HabitabilityState` that summarizes how habitable the planet is.
//!
//! HabitabilityModule evaluates the simulation but does not influence it.

use serde::{Deserialize, Serialize};
use worldsmith_models::{
    AtmosphereState, BiosphereState, CarbonCycleState, ClimateState, ClimateType, CryosphereState,
    HabitabilityClass, HabitabilityState, HydrologyState, LimitingFactor, Planet, PlanetId, SurfaceChemistryState,
};
use worldsmith_state::{FieldKey, SimulationEvent};
use worldsmith_traits::{ContractResult, ModuleContext, SimulationModule, StateWriter};

const DEFAULT_ATMOSPHERE_WEIGHT: f64 = 0.20;
const DEFAULT_CLIMATE_WEIGHT: f64 = 0.20;
const DEFAULT_WATER_WEIGHT: f64 = 0.20;
const DEFAULT_BIOSPHERE_WEIGHT: f64 = 0.15;
const DEFAULT_CHEMISTRY_WEIGHT: f64 = 0.15;
const DEFAULT_CRYOSPHERE_WEIGHT: f64 = 0.10;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HabitabilityConfig {
    pub atmosphere_weight: f64,
    pub climate_weight: f64,
    pub water_weight: f64,
    pub biosphere_weight: f64,
    pub chemistry_weight: f64,
    pub cryosphere_weight: f64,
}

impl Default for HabitabilityConfig {
    fn default() -> Self {
        Self {
            atmosphere_weight: DEFAULT_ATMOSPHERE_WEIGHT,
            climate_weight: DEFAULT_CLIMATE_WEIGHT,
            water_weight: DEFAULT_WATER_WEIGHT,
            biosphere_weight: DEFAULT_BIOSPHERE_WEIGHT,
            chemistry_weight: DEFAULT_CHEMISTRY_WEIGHT,
            cryosphere_weight: DEFAULT_CRYOSPHERE_WEIGHT,
        }
    }
}

pub struct HabitabilityModule {
    config: HabitabilityConfig,
    initialized: bool,
}

impl HabitabilityModule {
    pub fn new(config: HabitabilityConfig) -> Self {
        Self {
            config,
            initialized: false,
        }
    }

    fn atmosphere_suitability(&self, atmosphere: &AtmosphereState) -> f64 {
        let pressure = atmosphere.surface_pressure_pa;
        let pressure_score = if (50_000.0..=200_000.0).contains(&pressure) {
            1.0
        } else if pressure < 1.0 {
            0.0
        } else {
            let mid = 125_000.0;
            let half_width = 75_000.0;
            let dist = (pressure - mid).abs() / half_width;
            f64::clamp(1.0 - dist, 0.0, 1.0)
        };

        let co2 = atmosphere
            .atmosphere_composition
            .iter()
            .find(|g| g.molecule.formula == "CO2")
            .map(|g| g.abundance.value)
            .unwrap_or_default();
        let co2_score = if co2 < 0.01 {
            1.0
        } else if co2 > 0.1 {
            0.0
        } else {
            1.0
        };

        f64::clamp(pressure_score * 0.7 + co2_score * 0.3, 0.0, 1.0)
    }

    fn surface_habitability(&self, climate: &ClimateState) -> f64 {
        let temp = climate.equilibrium_temperature_k + climate.greenhouse_temperature_offset_k;
        if (250.0..=320.0).contains(&temp) {
            1.0
        } else if temp < 150.0 {
            0.0
        } else {
            let mid = 285.0;
            let half_width = 35.0;
            let dist = (temp - mid).abs() / half_width;
            f64::clamp(1.0 - dist, 0.0, 1.0)
        }
    }

    fn ocean_habitability(&self, hydro: &HydrologyState) -> f64 {
        f64::clamp(hydro.liquid_water_fraction, 0.0, 1.0)
    }

    fn biological_potential(&self, bio: &BiosphereState) -> f64 {
        let total_biomass_score = if bio.total_biomass_kg > 0.0 {
            f64::clamp(bio.total_biomass_kg.log10() / 18.0, 0.0, 1.0)
        } else {
            0.0
        };
        let productivity_score = if bio.productivity_rate_kg_per_s > 0.0 {
            f64::clamp(bio.productivity_rate_kg_per_s.log10() / 12.0, 0.0, 1.0)
        } else {
            0.0
        };
        f64::clamp(
            total_biomass_score * 0.6 + productivity_score * 0.4,
            0.0,
            1.0,
        )
    }

    fn climate_stability(&self, climate: &ClimateState) -> f64 {
        match climate.climate_classification {
            ClimateType::Temperate => 0.8,
            ClimateType::Tropical => 0.7,
            ClimateType::Frozen | ClimateType::Cold => 0.3,
            ClimateType::Arid => 0.2,
            _ => 0.1,
        }
    }

    fn water_availability(&self, hydro: &HydrologyState) -> f64 {
        f64::clamp(hydro.liquid_water_fraction, 0.0, 1.0)
    }

    fn cryosphere_penalty(&self, cryo: &CryosphereState) -> f64 {
        let ice = f64::clamp(cryo.planetary_ice_fraction, 0.0, 1.0);
        f64::clamp(1.0 - ice, 0.0, 1.0)
    }

    fn chemistry_score(&self, chem: &SurfaceChemistryState) -> f64 {
        f64::clamp(chem.mineral_availability, 0.0, 1.0)
    }

    fn classify(&self, overall: f64) -> HabitabilityClass {
        match overall {
            v if v >= 0.85 => HabitabilityClass::Paradise,
            v if v >= 0.65 => HabitabilityClass::HighlyHabitable,
            v if v >= 0.45 => HabitabilityClass::Habitable,
            v if v >= 0.25 => HabitabilityClass::Marginal,
            _ => HabitabilityClass::Hostile,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn limiting_factor(
        &self,
        atmo: f64,
        surface: f64,
        ocean: f64,
        bio: f64,
        climate: f64,
        water: f64,
        cryo: f64,
        chem: f64,
    ) -> LimitingFactor {
        let mut scores = [("Atmosphere", atmo),
            ("Climate", surface),
            ("Ocean", ocean),
            ("Biosphere", bio),
            ("ClimateStability", climate),
            ("Water", water),
            ("Cryosphere", cryo),
            ("Chemistry", chem)];
        scores.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        let weakest = scores[0].0;
        match weakest {
            "Atmosphere" => LimitingFactor::NoAtmosphere,
            "Climate" if surface < 0.3 => LimitingFactor::TooCold,
            "Climate" => LimitingFactor::TooHot,
            "Ocean" | "Water" => LimitingFactor::TooDry,
            "Biosphere" => LimitingFactor::LowBiomass,
            "Cryosphere" => LimitingFactor::GlobalIceCover,
            "Chemistry" => LimitingFactor::ExtremeCO2,
            _ => LimitingFactor::None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn tick(
        &self,
        mut habitability: HabitabilityState,
        atmosphere: &AtmosphereState,
        hydro: &HydrologyState,
        climate: &ClimateState,
        _carbon: &CarbonCycleState,
        bio: &BiosphereState,
        cryo: &CryosphereState,
        chem: &SurfaceChemistryState,
        _planet: &Planet,
    ) -> HabitabilityState {
        let atmo = self.atmosphere_suitability(atmosphere);
        let surface = self.surface_habitability(climate);
        let ocean = self.ocean_habitability(hydro);
        let bio_score = self.biological_potential(bio);
        let climate_stab = self.climate_stability(climate);
        let water = self.water_availability(hydro);
        let cryo = self.cryosphere_penalty(cryo);
        let chemistry = self.chemistry_score(chem);

        let total_weight = self.config.atmosphere_weight
            + self.config.climate_weight
            + self.config.water_weight
            + self.config.biosphere_weight
            + self.config.chemistry_weight
            + self.config.cryosphere_weight;
        let overall = if total_weight > 0.0 {
            (atmo * self.config.atmosphere_weight
                + surface * self.config.climate_weight
                + water * self.config.water_weight
                + bio_score * self.config.biosphere_weight
                + chemistry * self.config.chemistry_weight
                + cryo * self.config.cryosphere_weight)
                / total_weight
        } else {
            0.0
        };

        habitability.overall_habitability_index = f64::clamp(overall, 0.0, 1.0);
        habitability.surface_habitability_index = f64::clamp(surface, 0.0, 1.0);
        habitability.ocean_habitability_index = f64::clamp(ocean, 0.0, 1.0);
        habitability.biological_potential_index = f64::clamp(bio_score, 0.0, 1.0);
        habitability.climate_stability_index = f64::clamp(climate_stab, 0.0, 1.0);
        habitability.water_availability_index = f64::clamp(water, 0.0, 1.0);
        habitability.atmosphere_suitability_index = f64::clamp(atmo, 0.0, 1.0);
        habitability.habitability_class = self.classify(habitability.overall_habitability_index);
        habitability.limiting_factor = Some(self.limiting_factor(
            atmo,
            surface,
            ocean,
            bio_score,
            climate_stab,
            water,
            cryo,
            chemistry,
        ));
        habitability
    }
}

impl Default for HabitabilityModule {
    fn default() -> Self {
        Self::new(HabitabilityConfig::default())
    }
}

impl SimulationModule for HabitabilityModule {
    fn id(&self) -> &'static str {
        "worldsmith.evolution.habitability"
    }

    fn initialize(&mut self, state: &mut dyn StateWriter) -> ContractResult<()> {
        let planet_ids: Vec<_> = state.world().planets.keys().cloned().collect();
        for planet_id in planet_ids {
            if let Some(planet) = state.world().planets.get(&planet_id).cloned() {
                let habitability = HabitabilityState::default();
                let mut updated = planet;
                updated.habitability_state = Some(habitability);
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

        let snapshot: Vec<(PlanetId, Planet, Option<HabitabilityState>)> = state
            .world()
            .planets
            .values()
            .map(|planet| (planet.id, planet.clone(), planet.habitability_state.clone()))
            .collect();

        for (_planet_id, planet, habitability) in snapshot {
            let habitability = match habitability {
                Some(habitability) => habitability,
                None => continue,
            };

            let atmosphere = match &planet.atmosphere_state {
                Some(c) => c,
                None => continue,
            };
            let hydro = match &planet.hydrology_state {
                Some(h) => h,
                None => continue,
            };
            let climate = match &planet.climate_state {
                Some(c) => c,
                None => continue,
            };
            let carbon = match &planet.carbon_cycle_state {
                Some(c) => c,
                None => continue,
            };
            let bio = match &planet.biosphere_state {
                Some(b) => b,
                None => continue,
            };
            let cryo = match &planet.cryosphere_state {
                Some(c) => c,
                None => continue,
            };
            let chem = match &planet.surface_chemistry_state {
                Some(c) => c,
                None => continue,
            };

            let updated = self.tick(
                habitability,
                atmosphere,
                hydro,
                climate,
                carbon,
                bio,
                cryo,
                chem,
                &planet,
            );
            let mut updated_planet = planet;
            updated_planet.habitability_state = Some(updated);
            state
                .world_mut()
                .planets
                .insert(updated_planet.id, updated_planet);
        }

        Ok(())
    }

    fn reads(&self) -> Vec<FieldKey> {
        vec![
            FieldKey::OverallHabitabilityIndex,
            FieldKey::SurfaceHabitabilityIndex,
            FieldKey::OceanHabitabilityIndex,
            FieldKey::BiologicalPotentialIndex,
            FieldKey::ClimateStabilityIndex,
            FieldKey::WaterAvailabilityIndex,
            FieldKey::AtmosphereSuitabilityIndex,
            FieldKey::HabitabilityClass,
            FieldKey::LimitingFactor,
        ]
    }

    fn writes(&self) -> Vec<FieldKey> {
        vec![
            FieldKey::OverallHabitabilityIndex,
            FieldKey::SurfaceHabitabilityIndex,
            FieldKey::OceanHabitabilityIndex,
            FieldKey::BiologicalPotentialIndex,
            FieldKey::ClimateStabilityIndex,
            FieldKey::WaterAvailabilityIndex,
            FieldKey::AtmosphereSuitabilityIndex,
            FieldKey::HabitabilityClass,
            FieldKey::LimitingFactor,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HabitabilityModuleDiagnostics {
    pub overall: f64,
    pub class: HabitabilityClass,
    pub limiting: LimitingFactor,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AtmosphereModule, BiosphereModule, CarbonCycleModule, ClimateModule, CoreEvolutionModule,
        CryosphereModule, HydrologyModule, MantleEvolutionModule, PlateTectonicsModule,
        SurfaceChemistryModule, VolcanismModule,
    };
    use worldsmith_engine::EngineBuilder;
    use worldsmith_math::Vector3;
    use worldsmith_models::{
        AtmosphericGas, AtmosphericProperties, MeasuredValue, OceanProperties, OrbitalProperties,
        PhysicalProperties, PlanetClassificationState, Star, StarId, SystemId,
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
                climate_classification: ClimateType::Temperate,
            }),
            carbon_cycle_state: Some(CarbonCycleState::default()),
            biosphere_state: Some(BiosphereState::default()),
            cryosphere_state: Some(CryosphereState::default()),
            surface_chemistry_state: Some(SurfaceChemistryState::default()),
            habitability_state: Some(HabitabilityState::default()),
            classification_state: Some(PlanetClassificationState::default()),
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
        let module = HabitabilityModule::default();
        assert_eq!(module.id(), "worldsmith.evolution.habitability");
    }

    #[test]
    fn earth_like_planet_scores_highly() {
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
            .register_module(Box::new(HabitabilityModule::default()))
            .build()
            .unwrap();

        let planet_id = PlanetId(1);
        let star_id = StarId(1);
        let planet = earth_like_planet(planet_id, star_id);

        engine.state_mut().planets.insert(planet_id, planet);
        engine.state_mut().stars.insert(star_id, default_star());
        let _ = engine.initialize();
        let _ = engine.tick(100.0);

        let updated = engine
            .state()
            .planets
            .get(&planet_id)
            .and_then(|p| p.habitability_state.clone())
            .unwrap();

        assert!(
            updated.overall_habitability_index >= 0.5,
            "Earth-like planet should score above 0.5, got {}",
            updated.overall_habitability_index
        );
    }

    #[test]
    fn frozen_planet_scores_poorly() {
        let mut engine = EngineBuilder::new()
            .with_seed(7)
            .register_module(Box::new(HabitabilityModule::default()))
            .build()
            .unwrap();

        let planet_id = PlanetId(1);
        let star_id = StarId(1);
        let mut planet = earth_like_planet(planet_id, star_id);
        planet.climate_state = Some(ClimateState {
            equilibrium_temperature_k: 220.0,
            greenhouse_temperature_offset_k: 0.0,
            planetary_albedo: 0.6,
            climate_classification: ClimateType::Frozen,
        });
        planet.cryosphere_state = Some(CryosphereState {
            continental_ice_mass_kg: 3.0e22,
            sea_ice_mass_kg: 1.0e19,
            snow_mass_kg: 5.0e18,
            permanent_ice_fraction: 0.8,
            seasonal_snow_fraction: 0.2,
            melt_rate_kg_per_s: 0.0,
            freeze_rate_kg_per_s: 1.0e9,
            planetary_ice_fraction: 0.7,
            cryosphere_albedo_modifier: 0.4,
            sea_level_offset_m: -120.0,
        });

        engine.state_mut().planets.insert(planet_id, planet);
        engine.state_mut().stars.insert(star_id, default_star());
        let _ = engine.initialize();
        let _ = engine.tick(100.0);

        let updated = engine
            .state()
            .planets
            .get(&planet_id)
            .and_then(|p| p.habitability_state.clone())
            .unwrap();

        assert!(
            updated.overall_habitability_index < 0.5,
            "Frozen planet should score below 0.5, got {}",
            updated.overall_habitability_index
        );
    }

    #[test]
    fn hot_planet_scores_poorly() {
        let mut engine = EngineBuilder::new()
            .with_seed(7)
            .register_module(Box::new(HabitabilityModule::default()))
            .build()
            .unwrap();

        let planet_id = PlanetId(1);
        let star_id = StarId(1);
        let mut planet = earth_like_planet(planet_id, star_id);
        planet.climate_state = Some(ClimateState {
            equilibrium_temperature_k: 340.0,
            greenhouse_temperature_offset_k: 20.0,
            planetary_albedo: 0.2,
            climate_classification: ClimateType::Temperate,
        });

        engine.state_mut().planets.insert(planet_id, planet);
        engine.state_mut().stars.insert(star_id, default_star());
        let _ = engine.initialize();
        let _ = engine.tick(100.0);

        let updated = engine
            .state()
            .planets
            .get(&planet_id)
            .and_then(|p| p.habitability_state.clone())
            .unwrap();

        assert!(
            updated.overall_habitability_index < 0.5,
            "Hot planet should score below 0.5, got {}",
            updated.overall_habitability_index
        );
    }

    #[test]
    fn dry_planet_scores_poorly() {
        let mut engine = EngineBuilder::new()
            .with_seed(7)
            .register_module(Box::new(HabitabilityModule::default()))
            .build()
            .unwrap();

        let planet_id = PlanetId(1);
        let star_id = StarId(1);
        let mut planet = earth_like_planet(planet_id, star_id);
        planet.hydrology_state = Some(HydrologyState {
            total_water_mass_kg: 1.0e15,
            ocean_mass_kg: 0.0,
            atmospheric_water_mass_kg: 0.0,
            ice_mass_kg: 0.0,
            liquid_water_fraction: 0.0,
        });

        engine.state_mut().planets.insert(planet_id, planet);
        engine.state_mut().stars.insert(star_id, default_star());
        let _ = engine.initialize();
        let _ = engine.tick(100.0);

        let updated = engine
            .state()
            .planets
            .get(&planet_id)
            .and_then(|p| p.habitability_state.clone())
            .unwrap();

        assert!(
            updated.overall_habitability_index < 0.5,
            "Dry planet should score below 0.5, got {}",
            updated.overall_habitability_index
        );
    }

    #[test]
    fn habitability_does_not_modify_physical_states() {
        let mut engine = EngineBuilder::new()
            .with_seed(7)
            .register_module(Box::new(HabitabilityModule::default()))
            .build()
            .unwrap();

        let planet_id = PlanetId(1);
        let star_id = StarId(1);
        let planet = earth_like_planet(planet_id, star_id);
        let initial_climate = planet.climate_state.clone().unwrap();
        let initial_hydro = planet.hydrology_state.clone().unwrap();
        let initial_atmo = planet.atmosphere_state.clone().unwrap();

        engine.state_mut().planets.insert(planet_id, planet);
        engine.state_mut().stars.insert(star_id, default_star());
        let _ = engine.initialize();
        let _ = engine.tick(100.0);

        let updated = engine.state().planets.get(&planet_id).unwrap();

        assert_eq!(
            updated
                .climate_state
                .as_ref()
                .unwrap()
                .equilibrium_temperature_k,
            initial_climate.equilibrium_temperature_k,
        );
        assert_eq!(
            updated
                .hydrology_state
                .as_ref()
                .unwrap()
                .liquid_water_fraction,
            initial_hydro.liquid_water_fraction,
        );
        assert_eq!(
            updated
                .atmosphere_state
                .as_ref()
                .unwrap()
                .surface_pressure_pa,
            initial_atmo.surface_pressure_pa,
        );
    }

    #[test]
    fn snapshots_contain_habitability_state() {
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
            .register_module(Box::new(HabitabilityModule::default()))
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
            planet_snapshot.planet.habitability_state.is_some(),
            "snapshot must preserve habitability state"
        );
    }

    #[test]
    fn deterministic_replay() {
        let mut engine_a = EngineBuilder::new()
            .with_seed(42)
            .register_module(Box::new(HabitabilityModule::default()))
            .build()
            .unwrap();

        let mut engine_b = EngineBuilder::new()
            .with_seed(42)
            .register_module(Box::new(HabitabilityModule::default()))
            .build()
            .unwrap();

        let planet_id = PlanetId(1);
        let star_id = StarId(1);
        let mut planet = earth_like_planet(planet_id, star_id);
        planet.climate_state = Some(ClimateState {
            equilibrium_temperature_k: 280.0,
            greenhouse_temperature_offset_k: 10.0,
            planetary_albedo: 0.3,
            climate_classification: ClimateType::Temperate,
        });

        engine_a
            .state_mut()
            .planets
            .insert(planet_id, planet.clone());
        engine_a.state_mut().stars.insert(star_id, default_star());
        let _ = engine_a.initialize();
        let _ = engine_a.tick(100.0);

        engine_b.state_mut().planets.insert(planet_id, planet);
        engine_b.state_mut().stars.insert(star_id, default_star());
        let _ = engine_b.initialize();
        let _ = engine_b.tick(100.0);

        let state_a = engine_a.state().planets.get(&planet_id).unwrap();
        let state_b = engine_b.state().planets.get(&planet_id).unwrap();

        assert_eq!(
            state_a.habitability_state, state_b.habitability_state,
            "deterministic replay must produce identical habitability state"
        );
    }
}
