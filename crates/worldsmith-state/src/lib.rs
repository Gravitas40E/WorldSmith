//! Authoritative mutable simulation state and immutable snapshots.
//!
//! `WorldState` is the single source of truth for engine execution. Renderers,
//! UI layers, exporters, and tools should consume snapshots or events rather
//! than mutable state.

use std::collections::{BTreeMap, VecDeque};

use serde::{Deserialize, Serialize};
use worldsmith_models::{Moon, MoonId, Planet, PlanetId, Star, StarId, StellarSystem, SystemId};

/// Engine configuration stored with world state and save files.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct EngineConfig {
    /// Engine-level settings.
    pub engine: EngineSettings,
    /// Simulation timing settings.
    pub simulation: SimulationSettings,
    /// Rendering snapshot settings.
    pub rendering: RenderingSettings,
    /// Developer diagnostics settings.
    pub debug: DebugSettings,
}

/// Engine identity and deterministic seed settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EngineSettings {
    /// Human-readable engine instance name.
    pub name: String,
    /// Master deterministic seed.
    pub seed: u64,
    /// Optional future worker thread count.
    pub worker_threads: Option<usize>,
}

/// Simulation timestep settings.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SimulationSettings {
    /// Fixed timestep in seconds.
    pub fixed_timestep_seconds: f64,
    /// Maximum fixed substeps per outer tick.
    pub max_substeps: u32,
    /// Simulation speed multiplier.
    pub speed_multiplier: f64,
}

/// Rendering and snapshot settings.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RenderingSettings {
    /// Whether snapshot production for render consumers is enabled.
    pub enabled: bool,
    /// Snapshot interval in simulation seconds.
    pub snapshot_interval_seconds: f64,
}

/// Debug and diagnostics settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LogLevel {
    /// Verbose developer diagnostics.
    Debug,
    /// Normal informational diagnostics.
    Info,
    /// Recoverable warnings.
    Warning,
    /// Errors that need attention.
    Error,
}

/// Debug configuration.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DebugSettings {
    /// Enables extra diagnostics.
    pub enabled: bool,
    /// Minimum log level.
    pub log_level: LogLevel,
    /// Enables deterministic assertions in future systems.
    pub deterministic_assertions: bool,
}

impl Default for EngineSettings {
    fn default() -> Self {
        Self {
            name: "WorldSmith".to_string(),
            seed: 0,
            worker_threads: None,
        }
    }
}

impl Default for SimulationSettings {
    fn default() -> Self {
        Self {
            fixed_timestep_seconds: 1.0 / 60.0,
            max_substeps: 8,
            speed_multiplier: 1.0,
        }
    }
}

impl Default for RenderingSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            snapshot_interval_seconds: 1.0,
        }
    }
}

impl Default for DebugSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            log_level: LogLevel::Info,
            deterministic_assertions: true,
        }
    }
}

impl EngineConfig {
    /// Validates configuration values without mutating state.
    pub fn validate(&self) -> Result<(), String> {
        if self.engine.name.trim().is_empty() {
            return Err("engine name cannot be empty".to_string());
        }
        if self.simulation.fixed_timestep_seconds <= 0.0
            || !self.simulation.fixed_timestep_seconds.is_finite()
        {
            return Err("fixed timestep must be positive and finite".to_string());
        }
        if self.simulation.max_substeps == 0 {
            return Err("max_substeps must be greater than zero".to_string());
        }
        if self.simulation.speed_multiplier < 0.0 || !self.simulation.speed_multiplier.is_finite() {
            return Err("speed multiplier must be finite and non-negative".to_string());
        }
        if self.rendering.snapshot_interval_seconds <= 0.0
            || !self.rendering.snapshot_interval_seconds.is_finite()
        {
            return Err("snapshot interval must be positive and finite".to_string());
        }
        Ok(())
    }
}

/// Deterministic simulation clock stored in the authoritative state.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SimulationClock {
    elapsed_seconds: f64,
    accumulator_seconds: f64,
    fixed_timestep_seconds: f64,
    speed_multiplier: f64,
    paused: bool,
}

impl SimulationClock {
    /// Creates a clock with a fixed timestep.
    pub fn new(fixed_timestep_seconds: f64) -> Self {
        Self {
            elapsed_seconds: 0.0,
            accumulator_seconds: 0.0,
            fixed_timestep_seconds,
            speed_multiplier: 1.0,
            paused: false,
        }
    }

    /// Current simulation time in seconds.
    pub fn elapsed_seconds(&self) -> f64 {
        self.elapsed_seconds
    }

    /// Current fixed timestep in seconds.
    pub fn fixed_timestep_seconds(&self) -> f64 {
        self.fixed_timestep_seconds
    }

    /// Current speed multiplier.
    pub fn speed_multiplier(&self) -> f64 {
        self.speed_multiplier
    }

    /// Returns whether the clock is paused.
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Sets the simulation speed multiplier.
    pub fn set_speed_multiplier(&mut self, speed_multiplier: f64) {
        self.speed_multiplier = speed_multiplier.max(0.0);
    }

    /// Pauses simulation time.
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resumes simulation time.
    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// Advances variable timestep simulation time.
    pub fn advance_variable(&mut self, real_delta_seconds: f64) -> f64 {
        if self.paused || real_delta_seconds <= 0.0 {
            return 0.0;
        }
        let scaled = real_delta_seconds * self.speed_multiplier;
        self.elapsed_seconds += scaled;
        scaled
    }

    /// Accumulates real time for fixed timestep execution.
    pub fn accumulate_fixed(&mut self, real_delta_seconds: f64) {
        if !self.paused && real_delta_seconds > 0.0 {
            self.accumulator_seconds += real_delta_seconds * self.speed_multiplier;
        }
    }

    /// Consumes one fixed step if enough accumulated time exists.
    pub fn consume_fixed_step(&mut self) -> Option<f64> {
        if self.accumulator_seconds + f64::EPSILON < self.fixed_timestep_seconds {
            return None;
        }
        self.accumulator_seconds -= self.fixed_timestep_seconds;
        self.elapsed_seconds += self.fixed_timestep_seconds;
        Some(self.fixed_timestep_seconds)
    }

    /// Consumes up to `max_steps` fixed steps.
    pub fn consume_fixed_steps(&mut self, max_steps: u32) -> u32 {
        let mut steps = 0;
        while steps < max_steps && self.consume_fixed_step().is_some() {
            steps += 1;
        }
        steps
    }
}

impl Default for SimulationClock {
    fn default() -> Self {
        Self::new(1.0 / 60.0)
    }
}

/// Metadata describing a simulation run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimulationMetadata {
    /// Stable simulation identifier for save files, replays, and exports.
    pub simulation_id: String,
    /// Human-readable simulation name.
    pub name: String,
    /// Schema version used by serialized state.
    pub schema_version: u32,
    /// Creation timestamp as caller-provided text.
    pub created_at: Option<String>,
    /// Last modification timestamp as caller-provided text.
    pub updated_at: Option<String>,
    /// Free-form notes about this simulation.
    pub notes: Vec<String>,
}

impl Default for SimulationMetadata {
    fn default() -> Self {
        Self {
            simulation_id: "worldsmith-simulation".to_string(),
            name: "WorldSmith Simulation".to_string(),
            schema_version: 1,
            created_at: None,
            updated_at: None,
            notes: Vec::new(),
        }
    }
}

/// Strong event identifier assigned deterministically by the event queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EventId(pub u64);

/// Source of a state event.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventSource {
    /// Engine system source.
    Engine,
    /// Named simulation module.
    Module(String),
    /// Stellar system source.
    StellarSystem(SystemId),
    /// Star source.
    Star(StarId),
    /// Planet source.
    Planet(PlanetId),
    /// Moon source.
    Moon(MoonId),
}

/// Target of a state event.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventTarget {
    /// Event applies globally.
    Global,
    /// Event targets a stellar system.
    StellarSystem(SystemId),
    /// Event targets a star.
    Star(StarId),
    /// Event targets a planet.
    Planet(PlanetId),
    /// Event targets a moon.
    Moon(MoonId),
}

/// Deterministic event payload vocabulary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EventPayload {
    /// Star was created.
    StarCreated { star_id: StarId },
    /// Star data changed.
    StarUpdated { star_id: StarId },
    /// Stellar luminosity changed.
    LuminosityChanged { star_id: StarId },
    /// Star age advanced or was reassessed.
    StarAged { star_id: StarId },
    /// Stellar habitable zone changed.
    HabitableZoneChanged { star_id: StarId },
    /// Protoplanetary disk was created.
    DiskCreated { system_id: SystemId },
    /// Planetesimal was created.
    PlanetesimalCreated { local_id: u64 },
    /// Planetary embryo was created.
    EmbryoCreated { local_id: u64 },
    /// Planet migrated during formation.
    PlanetMigrated { planet_id: PlanetId },
    /// Collision or accretion event occurred.
    CollisionOccurred { embryo_id: u64, body_id: u64 },
    /// Planet classification changed or was assigned.
    PlanetClassificationChanged { planet_id: PlanetId },
    /// Planet interior differentiated into core, mantle, and crust.
    PlanetDifferentiated { planet_id: PlanetId },
    /// Planetary core formed.
    CoreFormed { planet_id: PlanetId },
    /// Magnetic field was generated.
    MagneticFieldGenerated { planet_id: PlanetId },
    /// Volcanism began or became active.
    VolcanismStarted { planet_id: PlanetId },
    /// Atmosphere was created.
    AtmosphereCreated { planet_id: PlanetId },
    /// Ocean or stable hydrosphere formed.
    OceanFormed { planet_id: PlanetId },
    /// Habitability assessment changed.
    HabitabilityChanged { planet_id: PlanetId },
    /// Planet was created.
    PlanetCreated { planet_id: PlanetId },
    /// Atmosphere data changed.
    AtmosphereChanged { planet_id: PlanetId },
    /// Surface temperature changed.
    SurfaceTemperatureChanged {
        planet_id: PlanetId,
        temperature_k: Option<f64>,
    },
    /// Volcanic eruption event.
    VolcanicEruption {
        body: EventTarget,
        intensity: Option<f64>,
    },
    /// Climate data changed.
    ClimateUpdated { planet_id: PlanetId },
    /// Magnetic field data changed.
    MagneticFieldChanged { planet_id: PlanetId },
    /// Orbital data changed.
    OrbitalChanged { target: EventTarget },
    /// Extensible typed event payload.
    Custom {
        kind: String,
        fields: Vec<(String, String)>,
    },
}

/// Immutable event consumed by future engine scheduling.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimulationEvent {
    /// Event identifier.
    pub id: EventId,
    /// Simulation timestamp in seconds.
    pub timestamp_s: f64,
    /// Event source.
    pub source: EventSource,
    /// Event target.
    pub target: EventTarget,
    /// Event payload.
    pub payload: EventPayload,
}

/// FIFO deterministic event queue.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventQueue {
    next_id: u64,
    events: VecDeque<SimulationEvent>,
}

impl EventQueue {
    /// Creates an empty event queue.
    pub fn new() -> Self {
        Self {
            next_id: 1,
            events: VecDeque::new(),
        }
    }

    /// Number of queued events.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Whether the queue has no events.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Queues a new immutable event and returns its assigned identifier.
    pub fn push(
        &mut self,
        timestamp_s: f64,
        source: EventSource,
        target: EventTarget,
        payload: EventPayload,
    ) -> EventId {
        let id = EventId(self.next_id);
        self.next_id += 1;
        self.events.push_back(SimulationEvent {
            id,
            timestamp_s,
            source,
            target,
            payload,
        });
        id
    }

    /// Pops the oldest queued event.
    pub fn pop(&mut self) -> Option<SimulationEvent> {
        self.events.pop_front()
    }

    /// Returns queued events in deterministic order.
    pub fn iter(&self) -> impl Iterator<Item = &SimulationEvent> {
        self.events.iter()
    }
}

impl Default for EventQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Strongly typed fields used for future read/write access declarations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FieldKey {
    /// Planet surface temperature in kelvin.
    SurfaceTemperature,
    /// Planet mass in kilograms.
    PlanetMass,
    /// Ocean coverage fraction.
    OceanCoverage,
    /// Atmospheric pressure in pascals.
    AtmosphericPressure,
    /// Magnetic field strength in tesla.
    MagneticFieldStrength,
    /// Orbital element state.
    OrbitalElements,
    /// Stellar luminosity in watts.
    StellarLuminosity,
    /// Surface gravity in meters per second squared.
    SurfaceGravity,
}

/// Registry entry describing a typed field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldDescriptor {
    /// Strong field key.
    pub key: FieldKey,
    /// Stable field name.
    pub name: String,
    /// Unit label, usually SI.
    pub unit: Option<String>,
    /// Human-readable description.
    pub description: String,
}

/// Field registry used by modules to declare read/write access.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldRegistry {
    fields: BTreeMap<FieldKey, FieldDescriptor>,
}

impl FieldRegistry {
    /// Creates a registry with WorldSmith's built-in fields.
    pub fn with_builtin_fields() -> Self {
        let mut registry = Self {
            fields: BTreeMap::new(),
        };
        registry.register(FieldDescriptor {
            key: FieldKey::SurfaceTemperature,
            name: "surface_temperature".to_string(),
            unit: Some("K".to_string()),
            description: "Representative planetary surface temperature".to_string(),
        });
        registry.register(FieldDescriptor {
            key: FieldKey::PlanetMass,
            name: "planet_mass".to_string(),
            unit: Some("kg".to_string()),
            description: "Planetary mass".to_string(),
        });
        registry.register(FieldDescriptor {
            key: FieldKey::OceanCoverage,
            name: "ocean_coverage".to_string(),
            unit: Some("fraction".to_string()),
            description: "Fractional ocean coverage".to_string(),
        });
        registry.register(FieldDescriptor {
            key: FieldKey::AtmosphericPressure,
            name: "atmospheric_pressure".to_string(),
            unit: Some("Pa".to_string()),
            description: "Surface atmospheric pressure".to_string(),
        });
        registry.register(FieldDescriptor {
            key: FieldKey::MagneticFieldStrength,
            name: "magnetic_field_strength".to_string(),
            unit: Some("T".to_string()),
            description: "Representative magnetic field strength".to_string(),
        });
        registry.register(FieldDescriptor {
            key: FieldKey::OrbitalElements,
            name: "orbital_elements".to_string(),
            unit: None,
            description: "Orbital element set for a celestial body".to_string(),
        });
        registry.register(FieldDescriptor {
            key: FieldKey::StellarLuminosity,
            name: "stellar_luminosity".to_string(),
            unit: Some("W".to_string()),
            description: "Stellar luminosity".to_string(),
        });
        registry.register(FieldDescriptor {
            key: FieldKey::SurfaceGravity,
            name: "surface_gravity".to_string(),
            unit: Some("m s^-2".to_string()),
            description: "Surface gravity".to_string(),
        });
        registry
    }

    /// Registers or replaces a field descriptor.
    pub fn register(&mut self, descriptor: FieldDescriptor) {
        self.fields.insert(descriptor.key, descriptor);
    }

    /// Looks up a field descriptor.
    pub fn get(&self, key: FieldKey) -> Option<&FieldDescriptor> {
        self.fields.get(&key)
    }

    /// Returns all registered fields in deterministic key order.
    pub fn iter(&self) -> impl Iterator<Item = &FieldDescriptor> {
        self.fields.values()
    }
}

impl Default for FieldRegistry {
    fn default() -> Self {
        Self::with_builtin_fields()
    }
}

/// Single mutable source of truth for a simulation run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldState {
    /// Simulation metadata.
    pub metadata: SimulationMetadata,
    /// Current deterministic seed.
    pub current_seed: u64,
    /// Simulation clock.
    pub clock: SimulationClock,
    /// Stellar systems by identifier.
    pub stellar_systems: BTreeMap<SystemId, StellarSystem>,
    /// Stars by identifier.
    pub stars: BTreeMap<StarId, Star>,
    /// Planets by identifier.
    pub planets: BTreeMap<PlanetId, Planet>,
    /// Moons by identifier.
    pub moons: BTreeMap<MoonId, Moon>,
    /// Deterministic event queue.
    pub event_queue: EventQueue,
    /// Engine configuration.
    pub engine_config: EngineConfig,
    /// Field registry.
    pub field_registry: FieldRegistry,
}

impl WorldState {
    /// Creates an empty state from an engine configuration.
    pub fn new(engine_config: EngineConfig) -> Self {
        let mut clock = SimulationClock::new(engine_config.simulation.fixed_timestep_seconds);
        clock.set_speed_multiplier(engine_config.simulation.speed_multiplier);
        Self {
            current_seed: engine_config.engine.seed,
            engine_config,
            metadata: SimulationMetadata::default(),
            clock,
            stellar_systems: BTreeMap::new(),
            stars: BTreeMap::new(),
            planets: BTreeMap::new(),
            moons: BTreeMap::new(),
            event_queue: EventQueue::new(),
            field_registry: FieldRegistry::default(),
        }
    }

    /// Produces an immutable simulation snapshot.
    pub fn snapshot(&self) -> SimulationSnapshot {
        SimulationSnapshot {
            metadata: self.metadata.clone(),
            timestamp_s: self.clock.elapsed_seconds(),
            stellar: StellarSnapshot {
                systems: self.stellar_systems.values().cloned().collect(),
                stars: self.stars.values().cloned().collect(),
            },
            planets: self
                .planets
                .values()
                .cloned()
                .map(PlanetSnapshot::from)
                .collect(),
            moons: self.moons.values().cloned().collect(),
        }
    }
}

impl Default for WorldState {
    fn default() -> Self {
        Self::new(EngineConfig::default())
    }
}

/// Immutable state snapshot for replay, export, and inspection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimulationSnapshot {
    /// Metadata captured with the snapshot.
    pub metadata: SimulationMetadata,
    /// Simulation timestamp in seconds.
    pub timestamp_s: f64,
    /// Stellar snapshot.
    pub stellar: StellarSnapshot,
    /// Planet snapshots.
    pub planets: Vec<PlanetSnapshot>,
    /// Moon snapshots.
    pub moons: Vec<Moon>,
}

/// Snapshot shaped for visualization systems.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VisualSnapshot {
    /// Simulation timestamp in seconds.
    pub timestamp_s: f64,
    /// Stellar snapshot.
    pub stellar: StellarSnapshot,
    /// Planet snapshots.
    pub planets: Vec<PlanetSnapshot>,
}

/// Immutable planet snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanetSnapshot {
    /// Strong planet identifier.
    pub id: PlanetId,
    /// Display name.
    pub name: String,
    /// Full planet data at snapshot time.
    pub planet: Planet,
}

impl From<Planet> for PlanetSnapshot {
    fn from(planet: Planet) -> Self {
        Self {
            id: planet.id,
            name: planet.name.clone(),
            planet,
        }
    }
}

/// Immutable stellar snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StellarSnapshot {
    /// Stellar systems.
    pub systems: Vec<StellarSystem>,
    /// Stars.
    pub stars: Vec<Star>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_receive_deterministic_ids() {
        let mut queue = EventQueue::new();
        let a = queue.push(
            0.0,
            EventSource::Engine,
            EventTarget::Global,
            EventPayload::Custom {
                kind: "a".to_string(),
                fields: Vec::new(),
            },
        );
        let b = queue.push(
            0.0,
            EventSource::Engine,
            EventTarget::Global,
            EventPayload::Custom {
                kind: "b".to_string(),
                fields: Vec::new(),
            },
        );
        assert_eq!(a, EventId(1));
        assert_eq!(b, EventId(2));
    }

    #[test]
    fn builtin_registry_has_core_fields() {
        let registry = FieldRegistry::default();
        assert!(registry.get(FieldKey::SurfaceTemperature).is_some());
        assert!(registry.get(FieldKey::AtmosphericPressure).is_some());
    }
}
