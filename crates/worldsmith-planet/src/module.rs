//! Engine integration for deterministic planet formation.

use serde::{Deserialize, Serialize};
use worldsmith_math::constants;
use worldsmith_models::StarId;
use worldsmith_state::{
    EventId, EventPayload, EventSource, EventTarget, FieldKey, SimulationEvent,
};
use worldsmith_traits::{
    ContractError, ContractResult, ModuleContext, SimulationModule, SnapshotProducer, StateWriter,
};

use crate::{
    builder::{FormationConfig, PlanetFormationBuilder, PlanetFormationOutput},
    disk::ProtoplanetaryDisk,
    evolution::{evolve_planet, PlanetEvolutionOutput},
};

/// Configuration for the planet formation module.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanetFormationModuleConfig {
    /// Source star id used to derive disk physics.
    pub parent_star_id: StarId,
    /// Formation settings.
    pub formation: FormationConfig,
    /// Disk age in megayears.
    pub disk_age_myr: f64,
}

impl Default for PlanetFormationModuleConfig {
    fn default() -> Self {
        Self {
            parent_star_id: StarId(1),
            formation: FormationConfig::default(),
            disk_age_myr: 1.0,
        }
    }
}

/// Simulation module that forms planets from stellar data and a protoplanetary disk.
pub struct PlanetFormationModule {
    config: PlanetFormationModuleConfig,
    output: Option<PlanetFormationOutput>,
    pending_events: Vec<SimulationEvent>,
    initialized: bool,
}

impl PlanetFormationModule {
    /// Creates a planet formation module.
    pub fn new(config: PlanetFormationModuleConfig) -> Self {
        Self {
            config,
            output: None,
            pending_events: Vec::new(),
            initialized: false,
        }
    }

    /// Returns the most recent formation output.
    pub fn output(&self) -> Option<&PlanetFormationOutput> {
        self.output.as_ref()
    }

    fn build_from_state(&self, state: &dyn StateWriter) -> ContractResult<PlanetFormationOutput> {
        let mut formation = self.config.formation.clone();
        formation.parent_star_id = self.config.parent_star_id;
        let star = state
            .world()
            .stars
            .get(&self.config.parent_star_id)
            .ok_or_else(|| {
                ContractError::InvalidInput(format!(
                    "missing parent star {:?}",
                    self.config.parent_star_id
                ))
            })?;
        let stellar_mass_solar = star.mass_kg.value / constants::SOLAR_MASS;
        let luminosity_solar = star.luminosity_w.value / constants::SOLAR_LUMINOSITY;
        let disk = ProtoplanetaryDisk::from_star(
            stellar_mass_solar,
            luminosity_solar,
            star.metallicity.value,
            self.config.disk_age_myr,
        );
        PlanetFormationBuilder::new()
            .seed(state.world().current_seed)
            .config(formation)
            .disk(disk)
            .build()
            .map_err(|err| ContractError::InvalidInput(err.to_string()))
    }

    fn queue_payloads(&mut self, timestamp_s: f64, payloads: &[EventPayload]) {
        let module_id = self.id().to_string();
        self.pending_events
            .extend(payloads.iter().cloned().map(|payload| SimulationEvent {
                id: EventId(0),
                timestamp_s,
                source: EventSource::Module(module_id.clone()),
                target: EventTarget::Global,
                payload,
            }));
    }
}

impl Default for PlanetFormationModule {
    fn default() -> Self {
        Self::new(PlanetFormationModuleConfig::default())
    }
}

impl SimulationModule for PlanetFormationModule {
    fn id(&self) -> &'static str {
        "worldsmith.planet_formation"
    }

    fn name(&self) -> &'static str {
        "WorldSmith Planet Formation Module"
    }

    fn initialize(&mut self, state: &mut dyn StateWriter) -> ContractResult<()> {
        let output = self.build_from_state(state)?;
        for planet in &output.planets {
            state.world_mut().planets.insert(planet.id, planet.clone());
        }
        self.queue_payloads(state.world().clock.elapsed_seconds(), &output.events);
        self.output = Some(output);
        self.initialized = true;
        Ok(())
    }

    fn update(
        &mut self,
        _context: ModuleContext,
        _state: &mut dyn StateWriter,
    ) -> ContractResult<()> {
        Ok(())
    }

    fn shutdown(&mut self, _state: &mut dyn StateWriter) -> ContractResult<()> {
        self.initialized = false;
        Ok(())
    }

    fn reads(&self) -> Vec<FieldKey> {
        vec![FieldKey::StellarLuminosity]
    }

    fn writes(&self) -> Vec<FieldKey> {
        vec![FieldKey::PlanetMass, FieldKey::OrbitalElements]
    }

    fn publish_events(&mut self) -> Vec<SimulationEvent> {
        std::mem::take(&mut self.pending_events)
    }

    fn consume_events(&mut self, events: &[SimulationEvent]) -> ContractResult<()> {
        for event in events {
            match &event.payload {
                EventPayload::StarUpdated { .. } => {
                    // Star data changed — may affect disk properties in future
                }
                EventPayload::LuminosityChanged { .. } => {
                    // Luminosity change may require disk or evolution recalculation
                }
                _ => {}
            }
        }
        Ok(())
    }
}

impl SnapshotProducer for PlanetFormationModule {
    type Snapshot = Option<PlanetFormationOutput>;

    fn snapshot(&self) -> Self::Snapshot {
        self.output.clone()
    }
}

/// Configuration for the planet evolution module.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanetEvolutionModuleConfig {
    /// Stellar luminosity fallback when parent star data is unavailable.
    pub fallback_luminosity_solar: f64,
    /// Evolution age in gigayears.
    pub age_gyr: f64,
}

impl Default for PlanetEvolutionModuleConfig {
    fn default() -> Self {
        Self {
            fallback_luminosity_solar: 1.0,
            age_gyr: 4.5,
        }
    }
}

/// Simulation module that evolves formed planets into geophysical worlds.
pub struct PlanetEvolutionModule {
    config: PlanetEvolutionModuleConfig,
    outputs: Vec<PlanetEvolutionOutput>,
    pending_events: Vec<SimulationEvent>,
    initialized: bool,
}

impl PlanetEvolutionModule {
    /// Creates a planet evolution module.
    pub fn new(config: PlanetEvolutionModuleConfig) -> Self {
        Self {
            config,
            outputs: Vec::new(),
            pending_events: Vec::new(),
            initialized: false,
        }
    }

    /// Returns latest evolution outputs.
    pub fn outputs(&self) -> &[PlanetEvolutionOutput] {
        &self.outputs
    }

    fn luminosity_for(&self, state: &dyn StateWriter) -> f64 {
        state
            .world()
            .stars
            .values()
            .next()
            .map(|star| star.luminosity_w.value / constants::SOLAR_LUMINOSITY)
            .unwrap_or(self.config.fallback_luminosity_solar)
    }

    fn queue_evolution_events(&mut self, timestamp_s: f64, output: &PlanetEvolutionOutput) {
        let module_id = self.id().to_string();
        let planet_id = output.planet.id;
        let mut payloads = vec![
            EventPayload::PlanetDifferentiated { planet_id },
            EventPayload::CoreFormed { planet_id },
            EventPayload::ClimateUpdated { planet_id },
            EventPayload::HabitabilityChanged { planet_id },
        ];
        if output.planet.magnetic_field.is_some() {
            payloads.push(EventPayload::MagneticFieldGenerated { planet_id });
        }
        if output
            .planet
            .geology
            .as_ref()
            .map(|g| g.volcanism != worldsmith_models::VolcanicActivity::None)
            .unwrap_or(false)
        {
            payloads.push(EventPayload::VolcanismStarted { planet_id });
        }
        if output.planet.atmosphere.is_some() {
            payloads.push(EventPayload::AtmosphereCreated { planet_id });
        }
        if output.planet.ocean.is_some() {
            payloads.push(EventPayload::OceanFormed { planet_id });
        }
        self.pending_events
            .extend(payloads.into_iter().map(|payload| SimulationEvent {
                id: EventId(0),
                timestamp_s,
                source: EventSource::Module(module_id.clone()),
                target: EventTarget::Planet(planet_id),
                payload,
            }));
    }
}

impl Default for PlanetEvolutionModule {
    fn default() -> Self {
        Self::new(PlanetEvolutionModuleConfig::default())
    }
}

impl SimulationModule for PlanetEvolutionModule {
    fn id(&self) -> &'static str {
        "worldsmith.planet_evolution"
    }

    fn name(&self) -> &'static str {
        "WorldSmith Planet Evolution Module"
    }

    fn initialize(&mut self, state: &mut dyn StateWriter) -> ContractResult<()> {
        let luminosity = self.luminosity_for(state);
        let planets = state.world().planets.values().cloned().collect::<Vec<_>>();
        self.outputs.clear();
        for planet in planets {
            let output = evolve_planet(planet, luminosity, self.config.age_gyr)
                .map_err(|err| ContractError::InvalidInput(err.to_string()))?;
            state
                .world_mut()
                .planets
                .insert(output.planet.id, output.planet.clone());
            self.queue_evolution_events(state.world().clock.elapsed_seconds(), &output);
            self.outputs.push(output);
        }
        self.initialized = true;
        Ok(())
    }

    fn update(
        &mut self,
        _context: ModuleContext,
        _state: &mut dyn StateWriter,
    ) -> ContractResult<()> {
        Ok(())
    }

    fn shutdown(&mut self, _state: &mut dyn StateWriter) -> ContractResult<()> {
        self.initialized = false;
        Ok(())
    }

    fn reads(&self) -> Vec<FieldKey> {
        vec![
            FieldKey::PlanetMass,
            FieldKey::OrbitalElements,
            FieldKey::StellarLuminosity,
        ]
    }

    fn writes(&self) -> Vec<FieldKey> {
        vec![
            FieldKey::SurfaceTemperature,
            FieldKey::AtmosphericPressure,
            FieldKey::OceanCoverage,
            FieldKey::MagneticFieldStrength,
            FieldKey::SurfaceGravity,
        ]
    }

    fn publish_events(&mut self) -> Vec<SimulationEvent> {
        std::mem::take(&mut self.pending_events)
    }

    fn consume_events(&mut self, events: &[SimulationEvent]) -> ContractResult<()> {
        for event in events {
            match &event.payload {
                EventPayload::SurfaceTemperatureChanged { .. } => {
                    // Surface temperature change — may trigger re-evolution
                }
                EventPayload::ClimateUpdated { .. } => {
                    // Climate updated by another module — re-evaluate feedback
                }
                EventPayload::OrbitalChanged { .. } => {
                    // Orbital change affects insolation — may need climate re-evaluation
                }
                EventPayload::LuminosityChanged { .. } => {
                    // Stellar luminosity change drives climate feedback
                }
                _ => {}
            }
        }
        Ok(())
    }
}

impl SnapshotProducer for PlanetEvolutionModule {
    type Snapshot = Vec<PlanetEvolutionOutput>;

    fn snapshot(&self) -> Self::Snapshot {
        self.outputs.clone()
    }
}
