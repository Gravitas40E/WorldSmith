//! Planetary classification: deterministic interpretation of simulation results.
//!
//! This module implements V1 deterministic planetary classification.
//! It reads outputs from all simulation modules and produces a single
//! `PlanetClassificationState` that categorizes the planet.
//!
//! PlanetClassificationModule interprets simulation results and never
//! influences planetary evolution.


use serde::{Deserialize, Serialize};
use worldsmith_models::{
    AtmosphereState, BiosphereCategory, BiosphereState, CarbonCycleState, ClimateState,
    ClimateType, CryosphereState, HabitabilityState, HydrologyState, HydrosphereCategory, Planet,
    PlanetClassificationState, PlanetId, PlanetType, PrimaryClassification,
    SecondaryClassification, SurfaceChemistryState,
};
use worldsmith_state::{FieldKey, SimulationEvent};
use worldsmith_traits::{ContractResult, ModuleContext, SimulationModule, StateWriter};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanetClassificationConfig {
    pub min_confidence: f64,
}

impl Default for PlanetClassificationConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.5,
        }
    }
}

pub struct PlanetClassificationModule {
    initialized: bool,
}

impl PlanetClassificationModule {
    pub fn new(_config: PlanetClassificationConfig) -> Self {
        Self {
            initialized: false,
        }
    }

    fn classify_primary(
        &self,
        planet: &Planet,
        hydro: &HydrologyState,
        cryo: &CryosphereState,
        climate: &ClimateState,
        chem: &SurfaceChemistryState,
    ) -> PrimaryClassification {
        let liquid = hydro.liquid_water_fraction;
        let ice = cryo.planetary_ice_fraction;
        let temp = climate.equilibrium_temperature_k + climate.greenhouse_temperature_offset_k;
        let silicate = chem.silicate_mass_kg;
        let reduced = chem.reduced_material_mass_kg;

        if ice > 0.6 && temp <= 220.0 {
            return PrimaryClassification::IceWorld;
        }
        if liquid > 0.65 && (250.0..=340.0).contains(&temp) {
            return PrimaryClassification::OceanWorld;
        }
        if temp > 900.0 && reduced > 0.0 {
            return PrimaryClassification::LavaWorld;
        }
        if silicate < 1.0e15 && liquid < 0.1 {
            return PrimaryClassification::DesertWorld;
        }
        if planet.planet_type == PlanetType::Rocky || planet.planet_type == PlanetType::Carbon {
            return PrimaryClassification::RockyPlanet;
        }
        PrimaryClassification::Terrestrial
    }

    fn classify_secondary(
        &self,
        climate: &ClimateState,
        _hydro: &HydrologyState,
        cryo: &CryosphereState,
        atmo: &AtmosphereState,
        carbon: &CarbonCycleState,
        bio: &BiosphereState,
    ) -> SecondaryClassification {
        let temp = climate.equilibrium_temperature_k + climate.greenhouse_temperature_offset_k;
        let ice = cryo.planetary_ice_fraction;
        let pressure = atmo.surface_pressure_pa;
        let biomass = bio.total_biomass_kg;

        let mut scores: Vec<(SecondaryClassification, f64)> = Vec::new();

        if matches!(
            climate.climate_classification,
            ClimateType::Temperate | ClimateType::Tropical
        ) && (250.0..=320.0).contains(&temp)
        {
            scores.push((SecondaryClassification::Temperate, 0.8));
        }
        if temp < 220.0 || ice > 0.5 {
            scores.push((SecondaryClassification::Frozen, 0.9));
        }
        if carbon.atmospheric_co2_fraction > 0.05 {
            scores.push((SecondaryClassification::CarbonRich, 0.85));
        }
        if biomass > 1.0e12 {
            scores.push((SecondaryClassification::HighBiomass, 0.8));
        }
        if pressure < 10_000.0 {
            scores.push((SecondaryClassification::LowAtmosphere, 0.9));
        }
        if pressure > 500_000.0 {
            scores.push((SecondaryClassification::DenseAtmosphere, 0.9));
        }

        if scores.is_empty() {
            return SecondaryClassification::None;
        }

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores[0].0
    }

    fn hydrosphere_category(&self, hydro: &HydrologyState) -> HydrosphereCategory {
        let liquid = hydro.liquid_water_fraction;
        let ice = hydro.ice_mass_kg / (hydro.total_water_mass_kg + 1e-9);

        match (liquid, ice) {
            (l, i) if l > 0.8 && i < 0.1 => HydrosphereCategory::Liquid,
            (l, i) if i > 0.8 && l < 0.1 => HydrosphereCategory::Ice,
            (_, _) if liquid < 0.05 && ice < 0.05 => HydrosphereCategory::Dry,
            _ => HydrosphereCategory::Mixed,
        }
    }

    fn biosphere_category(&self, bio: &BiosphereState) -> BiosphereCategory {
        let biomass = bio.total_biomass_kg;
        if biomass > 1.0e13 {
            BiosphereCategory::Dominant
        } else if biomass > 1.0e11 {
            BiosphereCategory::HighBiomass
        } else if biomass > 1.0e9 {
            BiosphereCategory::ModerateBiomass
        } else if biomass > 1.0e6 {
            BiosphereCategory::LowBiomass
        } else {
            BiosphereCategory::None
        }
    }

    fn compute_confidence(
        &self,
        primary: PrimaryClassification,
        _secondary: SecondaryClassification,
        _climate: &ClimateState,
        _hydro: &HydrologyState,
        _cryo: &CryosphereState,
    ) -> f64 {
        let confidence = match primary {
            PrimaryClassification::Terrestrial => 0.8,
            PrimaryClassification::OceanWorld => 0.85,
            PrimaryClassification::IceWorld => 0.85,
            PrimaryClassification::DesertWorld => 0.75,
            PrimaryClassification::LavaWorld => 0.7,
            PrimaryClassification::RockyPlanet => 0.8,
        };
        f64::clamp(confidence, 0.0, 1.0)
    }

    fn build_summary(&self, state: &PlanetClassificationState) -> String {
        let primary = match state.primary_classification {
            PrimaryClassification::Terrestrial => "Terrestrial",
            PrimaryClassification::OceanWorld => "Ocean World",
            PrimaryClassification::IceWorld => "Ice World",
            PrimaryClassification::DesertWorld => "Desert World",
            PrimaryClassification::LavaWorld => "Lava World",
            PrimaryClassification::RockyPlanet => "Rocky Planet",
        };
        let secondary = match state.secondary_classification {
            SecondaryClassification::None => String::new(),
            SecondaryClassification::Temperate => "Temperate".into(),
            SecondaryClassification::Frozen => "Frozen".into(),
            SecondaryClassification::CarbonRich => "Carbon-Rich".into(),
            SecondaryClassification::HighBiomass => "High-Biomass".into(),
            SecondaryClassification::LowAtmosphere => "Low-Atmosphere".into(),
            SecondaryClassification::DenseAtmosphere => "Dense-Atmosphere".into(),
        };
        let climate = match state.climate_category {
            ClimateType::Unknown => String::from("unknown climate"),
            ClimateType::Frozen => String::from("frozen climate"),
            ClimateType::Cold => String::from("cold climate"),
            ClimateType::Temperate => String::from("temperate climate"),
            ClimateType::Arid => String::from("arid climate"),
            ClimateType::Tropical => String::from("tropical climate"),
            ClimateType::Warm => String::from("warm climate"),
            ClimateType::Hot => String::from("hot climate"),
            ClimateType::Inferno => String::from("inferno-class temperatures"),
            ClimateType::RunawayGreenhouse => String::from("runaway greenhouse"),
            _ => String::from("unknown climate"),
        };

        if secondary.is_empty() {
            format!(
                "Classified as {} because planetary properties, hydrosphere, cryosphere, and climate match {} with {}.",
                primary, primary, climate
            )
        } else {
            format!(
                "Classified as {} {} because planetary properties, hydrosphere, cryosphere, and climate match {} and {} conditions.",
                secondary, primary, secondary, climate
            )
        }
    }

    fn build_notable_features(
        &self,
        atmo: &AtmosphereState,
        hydro: &HydrologyState,
        cryo: &CryosphereState,
        bio: &BiosphereState,
        habitability: Option<&HabitabilityState>,
    ) -> Vec<String> {
        let mut features = Vec::new();

        if atmo.surface_pressure_pa > 500_000.0 {
            features.push("Dense atmosphere".into());
        } else if atmo.surface_pressure_pa < 10_000.0 {
            features.push("Thin atmosphere".into());
        }

        if hydro.liquid_water_fraction > 0.65 {
            features.push("Surface liquid water".into());
        }
        if cryo.planetary_ice_fraction > 0.5 {
            features.push("Global ice cover".into());
        }
        if bio.total_biomass_kg > 1.0e12 {
            features.push("High biomass".into());
        }
        match habitability {
            Some(h) if h.habitability_class == worldsmith_models::HabitabilityClass::Paradise => {
                features.push("Highly habitable".into());
            }
            Some(h) if h.habitability_class == worldsmith_models::HabitabilityClass::Hostile => {
                features.push("Hostile environment".into());
            }
            _ => {}
        }
        features
    }

    #[allow(clippy::too_many_arguments)]
    fn tick(
        &self,
        mut classification: PlanetClassificationState,
        planet: &Planet,
        atmo: &AtmosphereState,
        hydro: &HydrologyState,
        climate: &ClimateState,
        carbon: &CarbonCycleState,
        bio: &BiosphereState,
        cryo: &CryosphereState,
        chem: &SurfaceChemistryState,
        habitability: Option<&HabitabilityState>,
    ) -> PlanetClassificationState {
        let primary = self.classify_primary(planet, hydro, cryo, climate, chem);
        let secondary = self.classify_secondary(climate, hydro, cryo, atmo, carbon, bio);
        let hydro_category = self.hydrosphere_category(hydro);
        let bio_category = self.biosphere_category(bio);
        let confidence = self.compute_confidence(primary, secondary, climate, hydro, cryo);

        classification.primary_classification = primary;
        classification.secondary_classification = secondary;
        classification.terrestrial_type = planet.planet_type;
        classification.climate_category = climate.climate_classification;
        classification.hydrosphere_category = hydro_category;
        classification.biosphere_category = bio_category;
        classification.confidence_score = confidence;
        classification.notable_features =
            self.build_notable_features(atmo, hydro, cryo, bio, habitability);
        classification.classification_summary = String::new(); // placeholder until after build

        classification.classification_summary = self.build_summary(&classification);
        classification
    }
}

impl Default for PlanetClassificationModule {
    fn default() -> Self {
        Self::new(PlanetClassificationConfig::default())
    }
}

impl SimulationModule for PlanetClassificationModule {
    fn id(&self) -> &'static str {
        "worldsmith.evolution.planet_classification"
    }

    fn initialize(&mut self, state: &mut dyn StateWriter) -> ContractResult<()> {
        let planet_ids: Vec<_> = state.world().planets.keys().cloned().collect();
        for planet_id in planet_ids {
            if let Some(planet) = state.world().planets.get(&planet_id).cloned() {
                let classification = PlanetClassificationState::default();
                let mut updated = planet;
                updated.classification_state = Some(classification);
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

        let snapshot: Vec<(PlanetId, Planet, Option<PlanetClassificationState>)> = state
            .world()
            .planets
            .values()
            .map(|planet| {
                (
                    planet.id,
                    planet.clone(),
                    planet.classification_state.clone(),
                )
            })
            .collect();

        for (_planet_id, planet, classification) in snapshot {
            let classification = match classification {
                Some(classification) => classification,
                None => continue,
            };

            let atmo = match &planet.atmosphere_state {
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
            let habitability = planet.habitability_state.as_ref();

            let updated = self.tick(
                classification,
                &planet,
                atmo,
                hydro,
                climate,
                carbon,
                bio,
                cryo,
                chem,
                habitability,
            );
            let mut updated_planet = planet;
            updated_planet.classification_state = Some(updated);
            state
                .world_mut()
                .planets
                .insert(updated_planet.id, updated_planet);
        }

        Ok(())
    }

    fn reads(&self) -> Vec<FieldKey> {
        vec![
            FieldKey::PrimaryClassification,
            FieldKey::SecondaryClassification,
            FieldKey::TerrestrialType,
            FieldKey::ClimateCategory,
            FieldKey::HydrosphereCategory,
            FieldKey::BiosphereCategory,
            FieldKey::ClassificationConfidence,
            FieldKey::ClassificationSummary,
            FieldKey::NotableFeatures,
        ]
    }

    fn writes(&self) -> Vec<FieldKey> {
        vec![
            FieldKey::PrimaryClassification,
            FieldKey::SecondaryClassification,
            FieldKey::TerrestrialType,
            FieldKey::ClimateCategory,
            FieldKey::HydrosphereCategory,
            FieldKey::BiosphereCategory,
            FieldKey::ClassificationConfidence,
            FieldKey::ClassificationSummary,
            FieldKey::NotableFeatures,
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
pub struct PlanetClassificationModuleDiagnostics {
    pub overall: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AtmosphereModule, BiosphereModule, CarbonCycleModule, ClimateModule, CoreEvolutionModule,
        CryosphereModule, HabitabilityModule, HydrologyModule, MantleEvolutionModule,
        PlateTectonicsModule, SurfaceChemistryModule, VolcanismModule,
    };
    use worldsmith_engine::EngineBuilder;
    use worldsmith_math::Vector3;
    use worldsmith_models::{
        AtmosphericGas, AtmosphericProperties, MeasuredValue, OceanProperties, OrbitalProperties,
        PhysicalProperties, Star, StarId, SystemId,
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
        let module = PlanetClassificationModule::default();
        assert_eq!(module.id(), "worldsmith.evolution.planet_classification");
    }

    #[test]
    fn earth_like_classifies() {
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
            .register_module(Box::new(PlanetClassificationModule::default()))
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
            .and_then(|p| p.classification_state.clone())
            .unwrap();

        assert!(
            matches!(
                updated.primary_classification,
                PrimaryClassification::OceanWorld | PrimaryClassification::Terrestrial
            ),
            "Earth-like planet should be Ocean World or Terrestrial, got {:?}",
            updated.primary_classification
        );
        assert!(
            updated.confidence_score >= 0.5,
            "confidence {} must exceed threshold {}",
            updated.confidence_score,
            0.5
        );
        assert!(!updated.classification_summary.is_empty());
    }

    #[test]
    fn frozen_world_classifies() {
        let mut engine = EngineBuilder::new()
            .with_seed(7)
            .register_module(Box::new(PlanetClassificationModule::default()))
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
        planet.hydrology_state = Some(HydrologyState {
            total_water_mass_kg: 1.4e21,
            ocean_mass_kg: 0.0,
            atmospheric_water_mass_kg: 0.0,
            ice_mass_kg: 1.4e21,
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
            .and_then(|p| p.classification_state.clone())
            .unwrap();

        assert_eq!(
            updated.primary_classification,
            PrimaryClassification::IceWorld
        );
    }

    #[test]
    fn desert_world_classifies() {
        let mut engine = EngineBuilder::new()
            .with_seed(7)
            .register_module(Box::new(PlanetClassificationModule::default()))
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
        planet.climate_state = Some(ClimateState {
            equilibrium_temperature_k: 310.0,
            greenhouse_temperature_offset_k: 10.0,
            planetary_albedo: 0.2,
            climate_classification: ClimateType::Arid,
        });
        planet.surface_chemistry_state = Some(SurfaceChemistryState {
            silicate_mass_kg: 0.0,
            ..Default::default()
        });

        engine.state_mut().planets.insert(planet_id, planet);
        engine.state_mut().stars.insert(star_id, default_star());
        let _ = engine.initialize();
        let _ = engine.tick(100.0);

        let updated = engine
            .state()
            .planets
            .get(&planet_id)
            .and_then(|p| p.classification_state.clone())
            .unwrap();

        assert_eq!(
            updated.primary_classification,
            PrimaryClassification::DesertWorld
        );
    }

    #[test]
    fn lava_world_classifies() {
        let mut engine = EngineBuilder::new()
            .with_seed(7)
            .register_module(Box::new(PlanetClassificationModule::default()))
            .build()
            .unwrap();

        let planet_id = PlanetId(1);
        let star_id = StarId(1);
        let mut planet = earth_like_planet(planet_id, star_id);
        planet.climate_state = Some(ClimateState {
            equilibrium_temperature_k: 1200.0,
            greenhouse_temperature_offset_k: 0.0,
            planetary_albedo: 0.1,
            climate_classification: ClimateType::Inferno,
        });
        planet.surface_chemistry_state = Some(SurfaceChemistryState {
            reduced_material_mass_kg: 1.0e18,
            ..Default::default()
        });

        engine.state_mut().planets.insert(planet_id, planet);
        engine.state_mut().stars.insert(star_id, default_star());
        let _ = engine.initialize();
        let _ = engine.tick(100.0);

        let updated = engine
            .state()
            .planets
            .get(&planet_id)
            .and_then(|p| p.classification_state.clone())
            .unwrap();

        assert_eq!(
            updated.primary_classification,
            PrimaryClassification::LavaWorld
        );
    }

    #[test]
    fn classification_does_not_modify_physical_states() {
        let mut engine = EngineBuilder::new()
            .with_seed(7)
            .register_module(Box::new(PlanetClassificationModule::default()))
            .build()
            .unwrap();

        let planet_id = PlanetId(1);
        let star_id = StarId(1);
        let planet = earth_like_planet(planet_id, star_id);
        let initial_climate = planet.climate_state.clone().unwrap();
        let initial_hydro = planet.hydrology_state.clone().unwrap();

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
    }

    #[test]
    fn snapshots_contain_classification_state() {
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
            .register_module(Box::new(PlanetClassificationModule::default()))
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
            planet_snapshot.planet.classification_state.is_some(),
            "snapshot must preserve classification state"
        );
    }

    #[test]
    fn deterministic_replay() {
        let mut engine_a = EngineBuilder::new()
            .with_seed(42)
            .register_module(Box::new(PlanetClassificationModule::default()))
            .build()
            .unwrap();

        let mut engine_b = EngineBuilder::new()
            .with_seed(42)
            .register_module(Box::new(PlanetClassificationModule::default()))
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
            state_a.classification_state, state_b.classification_state,
            "deterministic replay must produce identical classification state"
        );
    }
}
