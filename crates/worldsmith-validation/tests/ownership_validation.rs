//! Ownership validation tests.

use worldsmith_evolution::{
    CoreEvolutionModule, MantleEvolutionModule, PlateTectonicsModule, VolcanismModule,
};
use worldsmith_validation::{validate_field_ownership, OwnershipError};

#[test]
fn validate_field_ownership_passes_with_single_writers() {
    let core = CoreEvolutionModule::default();
    let mantle = MantleEvolutionModule::default();
    let volcanism = VolcanismModule::default();
    let plate = PlateTectonicsModule::default();

    let modules: Vec<(String, &dyn worldsmith_traits::SimulationModule)> = vec![
        ("worldsmith.evolution.core".to_string(), &core),
        ("worldsmith.evolution.mantle".to_string(), &mantle),
        ("worldsmith.evolution.volcanism".to_string(), &volcanism),
        ("worldsmith.evolution.plate_tectonics".to_string(), &plate),
    ];
    assert!(validate_field_ownership(&modules).is_ok());
}

#[test]
fn validate_field_ownership_detects_simulated_collision() {
    #[derive(Default)]
    struct FakeModule;
    impl worldsmith_traits::SimulationModule for FakeModule {
        fn id(&self) -> &'static str {
            "fake.module"
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
    let fake = FakeModule::default();
    let modules: Vec<(String, &dyn worldsmith_traits::SimulationModule)> = vec![
        ("worldsmith.evolution.mantle".to_string(), &mantle),
        ("fake.module".to_string(), &fake),
    ];
    let result = validate_field_ownership(&modules);
    assert!(result.is_err());
    match result.unwrap_err() {
        OwnershipError::DuplicateWriter { field, .. } => {
            assert_eq!(field, worldsmith_state::FieldKey::MantleTemperature);
        }
    }
}
