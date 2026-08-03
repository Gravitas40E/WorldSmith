use std::sync::Mutex;

use crate::model::types::{Planet as PublicPlanet, Snapshot as PublicSnapshot};
use worldsmith_engine::EngineBuilder;
use worldsmith_models::Planet;
use worldsmith_state::SimulationSnapshot;

pub struct WorldSmithEngine {
    engine: Mutex<Option<worldsmith_engine::Engine>>,
}

impl WorldSmithEngine {
    pub fn new(_seed: u64) -> Result<Self, String> {
        Ok(Self {
            engine: Mutex::new(None),
        })
    }

    pub fn generate_planet(
        &self,
        seed: u64,
        radius_m: f64,
        mass_kg: f64,
        _stellar_class: Option<String>,
        _initial_water_fraction: Option<f64>,
    ) -> Result<PublicPlanet, String> {
        let mut engine = EngineBuilder::new()
            .with_seed(seed)
            .build()
            .map_err(|e| format!("{e:?}"))?;

        let planet = build_planet(seed, radius_m, mass_kg);
        engine.state_mut().planets.insert(planet.id, planet.clone());
        engine.initialize().map_err(|e| format!("{e:?}"))?;
        engine.tick(0.0).map_err(|e| format!("{e:?}"))?;

        let mut guard = self.engine.lock().unwrap();
        *guard = Some(engine);

        Ok(extract_public_planet(planet))
    }

    pub fn tick(&self, ticks: u32) -> Result<PublicSnapshot, String> {
        let mut guard = self.engine.lock().unwrap();
        let engine = guard.as_mut().ok_or("engine not initialized")?;

        for _ in 0..ticks {
            engine.tick(0.0).map_err(|e| format!("{e:?}"))?;
        }

        let snapshot = engine.latest_snapshot().ok_or("no snapshot")?.clone();
        Ok(public_snapshot(snapshot))
    }

    pub fn snapshot(&self) -> Result<PublicSnapshot, String> {
        let guard = self.engine.lock().unwrap();
        let engine = guard.as_ref().ok_or("engine not initialized")?;
        let snapshot = engine.latest_snapshot().ok_or("no snapshot")?.clone();
        Ok(public_snapshot(snapshot))
    }

    pub fn export_json(&self) -> Result<String, String> {
        let snapshot = self.snapshot()?;
        serde_json::to_string_pretty(&snapshot).map_err(|e| e.to_string())
    }

    pub fn import_json(&self, json: &str) -> Result<PublicPlanet, String> {
        let snapshot: PublicSnapshot = serde_json::from_str(json).map_err(|e| e.to_string())?;
        snapshot
            .planets
            .into_iter()
            .next()
            .ok_or_else(|| "no planets".into())
    }

    pub fn planet_state(&self) -> Result<PublicPlanet, String> {
        let guard = self.engine.lock().unwrap();
        let engine = guard.as_ref().ok_or("engine not initialized")?;
        let state = engine.state();
        let planet = state.planets.values().next().ok_or("no planets")?;
        Ok(extract_public_planet(planet.clone()))
    }
}

fn public_snapshot(snapshot: SimulationSnapshot) -> PublicSnapshot {
    let planets = snapshot
        .planets
        .into_iter()
        .map(|p| extract_public_planet(p.planet))
        .collect();
    PublicSnapshot {
        simulation_id: snapshot.metadata.simulation_id.to_string(),
        timestamp_s: snapshot.timestamp_s,
        tick: 0,
        planets,
    }
}

fn extract_public_planet(planet: Planet) -> PublicPlanet {
    PublicPlanet {
        id: planet.id.0.to_string(),
        name: planet.name,
        class: format!("{:?}", planet.class),
        planet_type: format!("{:?}", planet.planet_type),
        radius_m: planet.physical.radius_m.value,
        mass_kg: planet.physical.mass_kg.value,
        gravity_m_s2: planet.physical.surface_gravity_m_s2.map(|v| v.value),
        stellar_class: Some("MainSequence".to_string()),
        temperature_k: planet
            .climate_state
            .as_ref()
            .map(|c| c.equilibrium_temperature_k + c.greenhouse_temperature_offset_k),
        pressure_pa: planet
            .atmosphere_state
            .as_ref()
            .map(|a| a.surface_pressure_pa),
        water_fraction: planet
            .hydrology_state
            .as_ref()
            .map(|h| h.liquid_water_fraction),
        ice_fraction: planet
            .cryosphere_state
            .as_ref()
            .map(|c| c.planetary_ice_fraction),
        atmospheric_mass_kg: planet
            .atmosphere_state
            .as_ref()
            .map(|a| a.atmospheric_mass_kg),
        mean_temperature_k: planet
            .atmosphere_state
            .as_ref()
            .map(|a| a.mean_temperature_k),
        equilibrium_temperature_k: planet
            .climate_state
            .as_ref()
            .map(|c| c.equilibrium_temperature_k),
        planetary_albedo: planet.climate_state.as_ref().map(|c| c.planetary_albedo),
        habitability_index: planet
            .habitability_state
            .as_ref()
            .map(|h| h.overall_habitability_index),
        habitability_class: planet
            .habitability_state
            .as_ref()
            .map(|h| format!("{:?}", h.habitability_class)),
        primary_classification: planet
            .classification_state
            .as_ref()
            .map(|c| format!("{:?}", c.primary_classification)),
        secondary_classification: planet
            .classification_state
            .as_ref()
            .map(|c| format!("{:?}", c.secondary_classification)),
        confidence_score: planet
            .classification_state
            .as_ref()
            .map(|c| c.confidence_score),
        classification_summary: planet
            .classification_state
            .as_ref()
            .map(|c| c.classification_summary.clone()),
        age_seconds: planet.interior.as_ref().map(|i| i.age_seconds),
        tick: None,
    }
}

fn build_planet(seed: u64, radius_m: f64, mass_kg: f64) -> Planet {
    Planet {
        id: worldsmith_models::PlanetId(seed),
        name: format!("World-{seed}"),
        class: worldsmith_models::PlanetClass::Terrestrial,
        planet_type: worldsmith_models::PlanetType::Rocky,
        system_id: worldsmith_models::SystemId(seed),
        physical: worldsmith_models::PhysicalProperties {
            mass_kg: worldsmith_models::MeasuredValue {
                value: mass_kg,
                unit: "kg".into(),
                provenance: None,
            },
            radius_m: worldsmith_models::MeasuredValue {
                value: radius_m,
                unit: "m".into(),
                provenance: None,
            },
            density_kg_m3: None,
            surface_gravity_m_s2: None,
        },
        orbit: worldsmith_models::OrbitalProperties {
            parent: worldsmith_models::BodyReference::Star(worldsmith_models::StarId(seed)),
            semi_major_axis_m: worldsmith_models::MeasuredValue {
                value: 1.496e11,
                unit: "m".into(),
                provenance: None,
            },
            semi_minor_axis_m: None,
            eccentricity: worldsmith_models::MeasuredValue {
                value: 0.0167,
                unit: "dimensionless".into(),
                provenance: None,
            },
            inclination_rad: worldsmith_models::MeasuredValue {
                value: 0.0,
                unit: "rad".into(),
                provenance: None,
            },
            orbital_period_s: None,
            rotation_period_s: None,
            axial_tilt_rad: None,
        },
        geology: None,
        atmosphere: None,
        atmosphere_state: Some(worldsmith_models::AtmosphereState {
            atmospheric_mass_kg: mass_kg * 1e-6,
            surface_pressure_pa: 101_325.0,
            mean_temperature_k: 288.0,
            atmosphere_composition: Vec::new(),
        }),
        hydrology_state: Some(worldsmith_models::HydrologyState {
            total_water_mass_kg: 0.0,
            ocean_mass_kg: 0.0,
            atmospheric_water_mass_kg: 0.0,
            ice_mass_kg: 0.0,
            liquid_water_fraction: 0.0,
        }),
        climate_state: Some(worldsmith_models::ClimateState {
            equilibrium_temperature_k: 255.0,
            greenhouse_temperature_offset_k: 33.0,
            planetary_albedo: 0.3,
            climate_classification: worldsmith_models::ClimateType::Temperate,
        }),
        carbon_cycle_state: None,
        biosphere_state: None,
        habitability_state: Some(worldsmith_models::HabitabilityState {
            overall_habitability_index: 0.0,
            surface_habitability_index: 0.0,
            ocean_habitability_index: 0.0,
            biological_potential_index: 0.0,
            climate_stability_index: 0.0,
            water_availability_index: 0.0,
            atmosphere_suitability_index: 0.0,
            habitability_class: worldsmith_models::HabitabilityClass::Hostile,
            limiting_factor: None,
        }),
        classification_state: Some(worldsmith_models::PlanetClassificationState {
            primary_classification: worldsmith_models::PrimaryClassification::Terrestrial,
            secondary_classification: worldsmith_models::SecondaryClassification::Temperate,
            terrestrial_type: worldsmith_models::PlanetType::Rocky,
            climate_category: worldsmith_models::ClimateType::Temperate,
            hydrosphere_category: worldsmith_models::HydrosphereCategory::Liquid,
            biosphere_category: worldsmith_models::BiosphereCategory::None,
            confidence_score: 1.0,
            classification_summary: String::new(),
            notable_features: Vec::new(),
        }),
        surface_chemistry_state: None,
        cryosphere_state: Some(worldsmith_models::CryosphereState {
            continental_ice_mass_kg: 0.0,
            sea_ice_mass_kg: 0.0,
            snow_mass_kg: 0.0,
            permanent_ice_fraction: 0.0,
            seasonal_snow_fraction: 0.0,
            melt_rate_kg_per_s: 0.0,
            freeze_rate_kg_per_s: 0.0,
            planetary_ice_fraction: 0.0,
            cryosphere_albedo_modifier: 0.0,
            sea_level_offset_m: 0.0,
        }),
        interior: None,
        volcanism: None,
        plate_tectonics: None,
        climate: None,
        ocean: None,
        magnetic_field: None,
        habitability: None,
        position_m: worldsmith_math::Vector3::ZERO,
        velocity_m_s: worldsmith_math::Vector3::ZERO,
        moons: Vec::new(),
    }
}
