//! Minimal developer sandbox for validating engine orchestration.

use worldsmith_engine::{EngineBuilder, EngineResult};
use worldsmith_state::{
    EventId, EventPayload, EventSource, EventTarget, FieldKey, SimulationEvent,
};
use worldsmith_traits::{ContractResult, ModuleContext, SimulationModule, StateWriter};

struct DummyModule {
    id: &'static str,
    updates: u64,
    events_seen: u64,
}

impl DummyModule {
    fn new(id: &'static str) -> Self {
        Self {
            id,
            updates: 0,
            events_seen: 0,
        }
    }
}

impl SimulationModule for DummyModule {
    fn id(&self) -> &'static str {
        self.id
    }

    fn initialize(&mut self, _state: &mut dyn StateWriter) -> ContractResult<()> {
        Ok(())
    }

    fn update(
        &mut self,
        context: ModuleContext,
        _state: &mut dyn StateWriter,
    ) -> ContractResult<()> {
        self.updates += 1;
        assert!(context.delta_seconds > 0.0);
        Ok(())
    }

    fn shutdown(&mut self, _state: &mut dyn StateWriter) -> ContractResult<()> {
        Ok(())
    }

    fn reads(&self) -> Vec<FieldKey> {
        Vec::new()
    }

    fn writes(&self) -> Vec<FieldKey> {
        Vec::new()
    }

    fn publish_events(&mut self) -> Vec<SimulationEvent> {
        vec![SimulationEvent {
            id: EventId(0),
            timestamp_s: 0.0,
            source: EventSource::Module(self.id.to_string()),
            target: EventTarget::Global,
            payload: EventPayload::Custom {
                kind: format!("{}_updated", self.id),
                fields: vec![("updates".to_string(), self.updates.to_string())],
            },
        }]
    }

    fn consume_events(&mut self, events: &[SimulationEvent]) -> ContractResult<()> {
        self.events_seen += events.len() as u64;
        Ok(())
    }
}

fn main() -> EngineResult<()> {
    let mut engine = EngineBuilder::new()
        .with_seed(42)
        .with_debug(true)
        .register_module(Box::new(DummyModule::new("stellar_dummy")))
        .register_module_with_stage(
            Box::new(DummyModule::new("planet_dummy")),
            10,
            vec!["stellar_dummy".to_string()],
        )
        .build()?;

    engine.initialize()?;
    for _ in 0..3 {
        engine.tick_fixed()?;
    }
    let diagnostics = engine.diagnostics();
    assert_eq!(
        diagnostics.active_pipeline,
        vec!["stellar_dummy".to_string(), "planet_dummy".to_string()]
    );
    assert_eq!(diagnostics.snapshot_count, 4);
    engine.shutdown()?;
    Ok(())
}
