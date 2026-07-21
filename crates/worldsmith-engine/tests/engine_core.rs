use worldsmith_engine::{EngineBuilder, EngineError};
use worldsmith_state::{
    EventId, EventPayload, EventSource, EventTarget, FieldKey, SimulationEvent,
};
use worldsmith_traits::{ContractResult, ModuleContext, SimulationModule, StateWriter};

#[derive(Debug)]
struct TestModule {
    id: &'static str,
    published_kind: &'static str,
    updates: u64,
    consumed_events: u64,
}

impl TestModule {
    fn new(id: &'static str, published_kind: &'static str) -> Self {
        Self {
            id,
            published_kind,
            updates: 0,
            consumed_events: 0,
        }
    }
}

impl SimulationModule for TestModule {
    fn id(&self) -> &'static str {
        self.id
    }

    fn initialize(&mut self, _state: &mut dyn StateWriter) -> ContractResult<()> {
        Ok(())
    }

    fn update(
        &mut self,
        _context: ModuleContext,
        _state: &mut dyn StateWriter,
    ) -> ContractResult<()> {
        self.updates += 1;
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
                kind: self.published_kind.to_string(),
                fields: vec![("update".to_string(), self.updates.to_string())],
            },
        }]
    }

    fn consume_events(&mut self, events: &[SimulationEvent]) -> ContractResult<()> {
        self.consumed_events += events.len() as u64;
        Ok(())
    }
}

#[test]
fn module_registration_and_pipeline_order_are_deterministic() {
    let engine = EngineBuilder::new()
        .register_module_with_stage(
            Box::new(TestModule::new("b", "b")),
            0,
            vec!["a".to_string()],
        )
        .register_module_with_stage(Box::new(TestModule::new("a", "a")), 10, Vec::new())
        .build()
        .unwrap();

    assert_eq!(
        engine.diagnostics().active_pipeline,
        vec!["a".to_string(), "b".to_string()]
    );
}

#[test]
fn duplicate_modules_are_rejected() {
    let err = EngineBuilder::new()
        .register_module(Box::new(TestModule::new("same", "a")))
        .register_module(Box::new(TestModule::new("same", "b")))
        .build()
        .err()
        .expect("duplicate modules should fail");

    assert!(matches!(err, EngineError::DuplicateModule(_)));
}

#[test]
fn missing_dependencies_are_rejected() {
    let err = EngineBuilder::new()
        .register_module_with_stage(
            Box::new(TestModule::new("module", "event")),
            0,
            vec!["missing".to_string()],
        )
        .build()
        .err()
        .expect("missing dependency should fail");

    assert!(matches!(err, EngineError::MissingDependency { .. }));
}

#[test]
fn circular_dependencies_are_rejected() {
    let err = EngineBuilder::new()
        .register_module_with_stage(
            Box::new(TestModule::new("a", "a")),
            0,
            vec!["b".to_string()],
        )
        .register_module_with_stage(
            Box::new(TestModule::new("b", "b")),
            0,
            vec!["a".to_string()],
        )
        .build()
        .err()
        .expect("circular dependency should fail");

    assert!(matches!(err, EngineError::CircularDependency(_)));
}

#[test]
fn fixed_ticks_produce_snapshots_and_dispatch_events() {
    let mut engine = EngineBuilder::new()
        .with_seed(123)
        .register_module(Box::new(TestModule::new("a", "a")))
        .register_module(Box::new(TestModule::new("b", "b")))
        .build()
        .unwrap();

    engine.initialize().unwrap();
    engine.tick_fixed().unwrap();
    engine.tick_fixed().unwrap();
    let diagnostics = engine.diagnostics();

    assert_eq!(diagnostics.tick_count, 2);
    assert_eq!(diagnostics.snapshot_count, 3);
    assert_eq!(diagnostics.queued_event_count, 0);
    assert_eq!(diagnostics.scheduler_stats.module_updates, 4);
    assert_eq!(diagnostics.scheduler_stats.events_dispatched, 4);
}

#[test]
fn same_seed_runs_same_diagnostics() {
    fn run() -> u64 {
        let mut engine = EngineBuilder::new()
            .with_seed(999)
            .register_module(Box::new(TestModule::new("a", "a")))
            .register_module_with_stage(
                Box::new(TestModule::new("b", "b")),
                5,
                vec!["a".to_string()],
            )
            .build()
            .unwrap();
        engine.initialize().unwrap();
        for _ in 0..4 {
            engine.tick_fixed().unwrap();
        }
        engine.state_fingerprint()
    }

    assert_eq!(run(), run());
}
