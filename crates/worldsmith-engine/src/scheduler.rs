//! Fixed timestep scheduler and event dispatch.

use serde::{Deserialize, Serialize};
use worldsmith_state::{EventSource, SimulationEvent};
use worldsmith_traits::ModuleContext;

use crate::{
    error::{EngineError, EngineResult},
    pipeline::Pipeline,
    registry::ModuleRegistry,
};

/// Statistics gathered while executing the scheduler.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SchedulerStats {
    /// Number of module update calls executed.
    pub module_updates: u64,
    /// Number of events dispatched to modules.
    pub events_dispatched: u64,
}

/// Deterministic fixed timestep scheduler.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Scheduler {
    stats: SchedulerStats,
}

impl Scheduler {
    /// Creates a scheduler.
    pub fn new() -> Self {
        Self {
            stats: SchedulerStats::default(),
        }
    }

    /// Returns accumulated scheduler statistics.
    pub fn stats(&self) -> SchedulerStats {
        self.stats
    }

    /// Executes one fixed simulation step.
    pub fn step(
        &mut self,
        state: &mut worldsmith_state::WorldState,
        registry: &mut ModuleRegistry,
        pipeline: &Pipeline,
        delta_seconds: f64,
    ) -> EngineResult<()> {
        let order = pipeline.execution_order().to_vec();
        for id in &order {
            let context = ModuleContext {
                timestamp_s: state.clock.elapsed_seconds(),
                delta_seconds,
                seed: state.current_seed,
            };
            let module = registry.get_mut(id).ok_or_else(|| {
                EngineError::Lifecycle(format!("module `{id}` is missing during update"))
            })?;
            module.module_mut().update(context, state)?;
            self.stats.module_updates += 1;
            let published = module.module_mut().publish_events();
            self.enqueue_published_events(state, id, published);
        }
        self.dispatch_events(state, registry, &order)?;
        Ok(())
    }

    fn enqueue_published_events(
        &self,
        state: &mut worldsmith_state::WorldState,
        module_id: &str,
        events: Vec<SimulationEvent>,
    ) {
        for event in events {
            state.event_queue.push(
                event.timestamp_s,
                EventSource::Module(module_id.to_string()),
                event.target,
                event.payload,
            );
        }
    }

    fn dispatch_events(
        &mut self,
        state: &mut worldsmith_state::WorldState,
        registry: &mut ModuleRegistry,
        order: &[String],
    ) -> EngineResult<()> {
        let mut events = Vec::new();
        while let Some(event) = state.event_queue.pop() {
            events.push(event);
        }
        if events.is_empty() {
            return Ok(());
        }
        self.stats.events_dispatched += events.len() as u64;
        for id in order {
            let module = registry.get_mut(id).ok_or_else(|| {
                EngineError::Lifecycle(format!("module `{id}` is missing during event dispatch"))
            })?;
            module.module_mut().consume_events(&events)?;
        }
        Ok(())
    }
}
