//! Builder for validated engine construction.

use worldsmith_state::{EngineConfig, WorldState};
use worldsmith_traits::SimulationModule;

use crate::{
    engine::Engine,
    error::{EngineError, EngineResult},
    pipeline::Pipeline,
    registry::{ModuleRegistry, RegisteredModule},
};

/// Builder responsible for constructing a validated [`Engine`].
///
/// Standard simulation stacks register modules in this order:
/// 1. `worldsmith.stellar` (star generation)
/// 2. `worldsmith.planet_formation` (planetesimal accretion)
/// 3. `worldsmith.planet_evolution` (geophysical/climate evolution)
/// 4. `worldsmith.orbital` (orbital dynamics, after any module that mutates orbital elements)
///
/// Use [`register_module_with_stage`](Self::register_module_with_stage) with
/// explicit priorities and dependencies to enforce this ordering. The
/// `SimulationSnapshot` produced by the engine contains the propagated
/// world-space positions for downstream consumers such as visualization.
#[derive(Default)]
pub struct EngineBuilder {
    config: EngineConfig,
    initial_state: Option<WorldState>,
    modules: Vec<RegisteredModule>,
}

impl EngineBuilder {
    /// Creates a builder with default configuration.
    pub fn new() -> Self {
        Self {
            config: EngineConfig::default(),
            initial_state: None,
            modules: Vec::new(),
        }
    }

    /// Sets the master deterministic seed.
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.config.engine.seed = seed;
        self
    }

    /// Replaces the engine configuration.
    pub fn with_config(mut self, config: EngineConfig) -> Self {
        self.config = config;
        self
    }

    /// Uses an existing initial world state.
    pub fn with_initial_state(mut self, state: WorldState) -> Self {
        self.config = state.engine_config.clone();
        self.initial_state = Some(state);
        self
    }

    /// Enables or disables debug diagnostics.
    pub fn with_debug(mut self, enabled: bool) -> Self {
        self.config.debug.enabled = enabled;
        self
    }

    /// Registers a module with default priority and no dependencies.
    pub fn register_module(mut self, module: Box<dyn SimulationModule>) -> Self {
        let priority = self.modules.len() as i32;
        self.modules
            .push(RegisteredModule::new(module, priority, Vec::new()));
        self
    }

    /// Registers a module with explicit priority and dependencies.
    pub fn register_module_with_stage(
        mut self,
        module: Box<dyn SimulationModule>,
        priority: i32,
        dependencies: Vec<String>,
    ) -> Self {
        self.modules
            .push(RegisteredModule::new(module, priority, dependencies));
        self
    }

    /// Validates configuration, registry, and pipeline dependencies, then builds an engine.
    pub fn build(self) -> EngineResult<Engine> {
        let Self {
            config,
            initial_state,
            modules,
        } = self;
        config
            .validate()
            .map_err(EngineError::InvalidConfiguration)?;
        let mut registry = ModuleRegistry::new();
        for module in modules {
            registry.register(module)?;
        }
        let pipeline = Pipeline::build(registry.descriptors())?;
        let state = initial_state.unwrap_or_else(|| WorldState::new(config));
        if state.engine_config.validate().is_err() {
            return Err(EngineError::InvalidConfiguration(
                "initial world state contains invalid engine configuration".to_string(),
            ));
        }
        Ok(Engine::new(state, registry, pipeline))
    }
}
