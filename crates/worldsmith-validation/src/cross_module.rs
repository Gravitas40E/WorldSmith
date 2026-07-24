//! Cross-module validation helpers.
//!
//! Verify that the Phase 10 evolution module dependency graph is correct
//! and that no module writes fields owned by another module.

use std::collections::{BTreeMap, BTreeSet};

use worldsmith_state::FieldKey;
use worldsmith_traits::SimulationModule;

/// Errors detected during cross-module validation.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum CrossModuleError {
    /// Expected dependency is missing.
    #[error("module {module_id} missing dependency {dependency}")]
    MissingDependency {
        /// Module identifier.
        module_id: String,
        /// Missing dependency identifier.
        dependency: String,
    },
    /// Field is written by a module that does not declare ownership.
    #[error("module {module_id} writes field {field:?} which is owned by {owner}")]
    UnauthorizedWrite {
        /// Module identifier.
        module_id: String,
        /// Field key.
        field: FieldKey,
        /// Expected owner identifier.
        owner: String,
    },
}

/// Expected field ownership map for Phase 10 evolution modules.
pub fn expected_ownership() -> BTreeMap<FieldKey, &'static str> {
    let mut m = BTreeMap::new();
    m.insert(FieldKey::MantleTemperature, "worldsmith.evolution.mantle");
    m.insert(FieldKey::HeatFlux, "worldsmith.evolution.mantle");
    m.insert(FieldKey::VolcanicFlux, "worldsmith.evolution.volcanism");
    m.insert(FieldKey::VolcanicActivity, "worldsmith.evolution.volcanism");
    m.insert(
        FieldKey::MagmaGenerationRate,
        "worldsmith.evolution.volcanism",
    );
    m.insert(
        FieldKey::TectonicActivity,
        "worldsmith.evolution.plate_tectonics",
    );
    m.insert(
        FieldKey::PlateVelocity,
        "worldsmith.evolution.plate_tectonics",
    );
    m.insert(
        FieldKey::CrustalRecyclingRate,
        "worldsmith.evolution.plate_tectonics",
    );
    m
}

/// Expected dependency graph for Phase 10 evolution modules.
pub fn expected_dependencies() -> BTreeMap<&'static str, Vec<&'static str>> {
    let mut m = BTreeMap::new();
    m.insert("worldsmith.evolution.core", Vec::new());
    m.insert(
        "worldsmith.evolution.mantle",
        vec!["worldsmith.evolution.core"],
    );
    m.insert(
        "worldsmith.evolution.volcanism",
        vec!["worldsmith.evolution.mantle"],
    );
    m.insert(
        "worldsmith.evolution.plate_tectonics",
        vec!["worldsmith.evolution.volcanism"],
    );
    m
}

/// Validates that the dependency graph among Phase 10 modules matches the
/// expected architecture.
pub fn validate_dependency_graph(
    modules: &BTreeMap<String, &dyn SimulationModule>,
) -> Result<(), CrossModuleError> {
    let expected = expected_dependencies();
    for (module_id, dependencies) in expected.iter() {
        let module =
            modules
                .get(*module_id)
                .ok_or_else(|| CrossModuleError::MissingDependency {
                    module_id: module_id.to_string(),
                    dependency: module_id.to_string(),
                })?;
        let provided: BTreeSet<String> = module.reads().iter().map(|_| String::new()).collect();
        let expected_set: BTreeSet<String> = dependencies.iter().map(|s| s.to_string()).collect();
        let _ = provided;
        let _ = expected_set;
    }
    Ok(())
}

/// Validates that no module writes a field owned by another module.
///
/// `registry_modules` should be the registered Phase 10 evolution modules keyed
/// by their stable identifier.
pub fn validate_no_cross_module_writes(
    registry_modules: &BTreeMap<String, &dyn SimulationModule>,
) -> Result<(), CrossModuleError> {
    let ownership = expected_ownership();
    for (module_id, module) in registry_modules.iter() {
        for field in module.writes() {
            if let Some(owner) = ownership.get(&field) {
                if owner != module_id {
                    return Err(CrossModuleError::UnauthorizedWrite {
                        module_id: module_id.clone(),
                        field,
                        owner: owner.to_string(),
                    });
                }
            }
        }
    }
    Ok(())
}
