//! Cross-module validation tests.

use worldsmith_evolution::{
    CoreEvolutionModule, MantleEvolutionModule, PlateTectonicsModule, VolcanismModule,
};
use worldsmith_validation::{
    cross_module::CrossModuleError, validate_dependency_graph, validate_no_cross_module_writes,
};

#[test]
fn validate_dependency_graph_accepts_registered_modules() {
    let core = CoreEvolutionModule::default();
    let mantle = MantleEvolutionModule::default();
    let volcanism = VolcanismModule::default();
    let plate = PlateTectonicsModule::default();

    let mut modules =
        std::collections::BTreeMap::<String, &dyn worldsmith_traits::SimulationModule>::new();
    modules.insert("worldsmith.evolution.core".to_string(), &core);
    modules.insert("worldsmith.evolution.mantle".to_string(), &mantle);
    modules.insert("worldsmith.evolution.volcanism".to_string(), &volcanism);
    modules.insert("worldsmith.evolution.plate_tectonics".to_string(), &plate);
    assert!(validate_dependency_graph(&modules).is_ok());
}

#[test]
fn validate_no_cross_module_writes_passes_with_clean_ownership() {
    let mantle = MantleEvolutionModule::default();
    let volcanism = VolcanismModule::default();
    let plate = PlateTectonicsModule::default();

    let mut modules =
        std::collections::BTreeMap::<String, &dyn worldsmith_traits::SimulationModule>::new();
    modules.insert("worldsmith.evolution.mantle".to_string(), &mantle);
    modules.insert("worldsmith.evolution.volcanism".to_string(), &volcanism);
    modules.insert("worldsmith.evolution.plate_tectonics".to_string(), &plate);
    assert!(validate_no_cross_module_writes(&modules).is_ok());
}

#[test]
fn validate_no_cross_module_writes_detects_simulated_unauthorized_write() {
    #[derive(Default)]
    struct BadModule;
    impl worldsmith_traits::SimulationModule for BadModule {
        fn id(&self) -> &'static str {
            "bad.module"
        }
        fn initialize(
            &mut self,
            _state: &mut dyn worldsmith_traits::StateWriter,
        ) -> Result<(), worldsmith_traits::ContractError> {
            Ok(())
        }
        fn update(
            &mut self,
            _context: worldsmith_traits::ModuleContext,
            _state: &mut dyn worldsmith_traits::StateWriter,
        ) -> Result<(), worldsmith_traits::ContractError> {
            Ok(())
        }
        fn shutdown(
            &mut self,
            _state: &mut dyn worldsmith_traits::StateWriter,
        ) -> Result<(), worldsmith_traits::ContractError> {
            Ok(())
        }
        fn reads(&self) -> Vec<worldsmith_state::FieldKey> {
            Vec::new()
        }
        fn writes(&self) -> Vec<worldsmith_state::FieldKey> {
            vec![worldsmith_state::FieldKey::MantleTemperature]
        }
        fn publish_events(&mut self) -> Vec<worldsmith_state::SimulationEvent> {
            Vec::new()
        }
        fn consume_events(
            &mut self,
            _events: &[worldsmith_state::SimulationEvent],
        ) -> Result<(), worldsmith_traits::ContractError> {
            Ok(())
        }
    }

    let mantle = MantleEvolutionModule::default();
    let bad = BadModule::default();

    let mut modules =
        std::collections::BTreeMap::<String, &dyn worldsmith_traits::SimulationModule>::new();
    modules.insert("worldsmith.evolution.mantle".to_string(), &mantle);
    modules.insert("bad.module".to_string(), &bad);
    let result = validate_no_cross_module_writes(&modules);
    assert!(result.is_err());
    match result.unwrap_err() {
        CrossModuleError::UnauthorizedWrite { owner, field, .. } => {
            assert_eq!(owner, "worldsmith.evolution.mantle");
            assert_eq!(field, worldsmith_state::FieldKey::MantleTemperature);
        }
        _ => panic!("expected UnauthorizedWrite"),
    }
}
