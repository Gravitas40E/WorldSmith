//! Main WorldSmith engine runtime.

use worldsmith_rng::RngStream;
use worldsmith_state::{EngineConfig, SimulationSnapshot, WorldState};

use crate::{
    diagnostics::EngineDiagnostics,
    error::{EngineError, EngineResult},
    pipeline::Pipeline,
    registry::ModuleRegistry,
    scheduler::Scheduler,
};

/// Runtime responsible for orchestrating registered simulation modules.
pub struct Engine {
    state: WorldState,
    registry: ModuleRegistry,
    pipeline: Pipeline,
    scheduler: Scheduler,
    snapshots: Vec<SimulationSnapshot>,
    master_rng: RngStream,
    tick_count: u64,
    initialized: bool,
    running: bool,
}

impl Engine {
    /// Creates an engine from validated parts.
    pub(crate) fn new(state: WorldState, registry: ModuleRegistry, pipeline: Pipeline) -> Self {
        let seed = state.current_seed;
        Self {
            state,
            registry,
            pipeline,
            scheduler: Scheduler::new(),
            snapshots: Vec::new(),
            master_rng: RngStream::new(seed),
            tick_count: 0,
            initialized: false,
            running: false,
        }
    }

    /// Initializes all modules in deterministic pipeline order.
    pub fn initialize(&mut self) -> EngineResult<()> {
        if self.initialized {
            return Ok(());
        }
        self.state
            .engine_config
            .validate()
            .map_err(EngineError::InvalidConfiguration)?;
        let order = self.pipeline.execution_order().to_vec();
        self.registry.initialize(&order, &mut self.state)?;
        self.initialized = true;
        self.running = true;
        self.capture_snapshot();
        Ok(())
    }

    /// Advances the engine by real elapsed time using fixed substeps.
    pub fn tick(&mut self, real_delta_seconds: f64) -> EngineResult<u32> {
        if !self.initialized {
            return Err(EngineError::Lifecycle(
                "engine must be initialized before ticking".to_string(),
            ));
        }
        if !self.running || self.state.clock.is_paused() {
            return Ok(0);
        }

        self.tick_count += 1;
        self.state.clock.accumulate_fixed(real_delta_seconds);
        let max_steps = self.state.engine_config.simulation.max_substeps;
        let mut steps = 0;
        while steps < max_steps {
            let Some(delta_seconds) = self.state.clock.consume_fixed_step() else {
                break;
            };
            self.scheduler.step(
                &mut self.state,
                &mut self.registry,
                &self.pipeline,
                delta_seconds,
            )?;
            self.capture_snapshot();
            steps += 1;
        }
        Ok(steps)
    }

    /// Executes exactly one fixed timestep regardless of accumulated real time.
    pub fn tick_fixed(&mut self) -> EngineResult<()> {
        let delta = self.state.clock.fixed_timestep_seconds();
        self.state.clock.accumulate_fixed(delta);
        self.tick(0.0)?;
        Ok(())
    }

    /// Shuts down modules in reverse deterministic order.
    pub fn shutdown(&mut self) -> EngineResult<()> {
        if !self.initialized {
            self.running = false;
            return Ok(());
        }
        let order = self.pipeline.execution_order().to_vec();
        self.registry.shutdown(&order, &mut self.state)?;
        self.running = false;
        self.initialized = false;
        Ok(())
    }

    /// Returns whether the simulation loop should continue.
    pub fn running(&self) -> bool {
        self.running
    }

    /// Pauses simulation time.
    pub fn pause(&mut self) {
        self.state.clock.pause();
    }

    /// Resumes simulation time.
    pub fn resume(&mut self) {
        self.state.clock.resume();
    }

    /// Sets the deterministic simulation speed multiplier.
    pub fn set_speed_multiplier(&mut self, speed_multiplier: f64) {
        self.state.clock.set_speed_multiplier(speed_multiplier);
    }

    /// Returns immutable access to authoritative state.
    pub fn state(&self) -> &WorldState {
        &self.state
    }

    /// Returns mutable access to authoritative state for engine-managed setup.
    pub fn state_mut(&mut self) -> &mut WorldState {
        &mut self.state
    }

    /// Returns the active configuration.
    pub fn config(&self) -> &EngineConfig {
        &self.state.engine_config
    }

    /// Returns retained immutable snapshots.
    pub fn snapshots(&self) -> &[SimulationSnapshot] {
        &self.snapshots
    }

    /// Returns the most recent snapshot.
    pub fn latest_snapshot(&self) -> Option<&SimulationSnapshot> {
        self.snapshots.last()
    }

    /// Derives a deterministic RNG stream for a module or subsystem.
    pub fn rng_stream(&self, label: &str) -> RngStream {
        self.master_rng.derive(label)
    }

    /// Returns developer diagnostics.
    pub fn diagnostics(&self) -> EngineDiagnostics {
        EngineDiagnostics {
            registered_modules: self.registry.ids(),
            active_pipeline: self.pipeline.execution_order().to_vec(),
            tick_count: self.tick_count,
            simulation_time_s: self.state.clock.elapsed_seconds(),
            queued_event_count: self.state.event_queue.len(),
            snapshot_count: self.snapshots.len(),
            current_seed: self.state.current_seed,
            scheduler_stats: self.scheduler.stats(),
            initialized: self.initialized,
            running: self.running,
        }
    }

    /// Produces a deterministic lightweight fingerprint for repeatability tests.
    ///
    /// This is not a cryptographic hash and is not a save-file checksum. It is
    /// intended for tests and diagnostics that need to compare execution shape.
    pub fn state_fingerprint(&self) -> u64 {
        let mut hash = 0xcbf2_9ce4_8422_2325u64;
        hash = fnv1a(hash, self.state.current_seed);
        hash = fnv1a(hash, self.tick_count);
        hash = fnv1a(hash, self.snapshots.len() as u64);
        hash = fnv1a(hash, self.state.event_queue.len() as u64);
        hash = fnv1a(hash, self.state.clock.elapsed_seconds().to_bits());
        for id in self.pipeline.execution_order() {
            for byte in id.as_bytes() {
                hash = fnv1a(hash, *byte as u64);
            }
        }
        hash
    }

    fn capture_snapshot(&mut self) {
        if self.state.engine_config.rendering.enabled {
            self.snapshots.push(self.state.snapshot());
        }
    }
}

fn fnv1a(hash: u64, value: u64) -> u64 {
    (hash ^ value).wrapping_mul(0x0000_0100_0000_01B3)
}
