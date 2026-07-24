//! Ownership validation helpers.
//!
//! Verify that every field has exactly one declared runtime writer by
//! inspecting `reads()` / `writes()` on registered `SimulationModule`s.

use std::collections::BTreeMap;

use worldsmith_state::FieldKey;
use worldsmith_traits::SimulationModule;

/// Errors detected during ownership validation.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum OwnershipError {
    /// Field is written by more than one module.
    #[error("field {field:?} has duplicate writers: {writers:?}")]
    DuplicateWriter {
        /// Field key.
        field: FieldKey,
        /// Set of module identifiers.
        writers: Vec<String>,
    },
}

/// Validates that each `FieldKey` has at most one writer among the provided
/// modules.
///
/// Returns a list of module declarations for diagnostic purposes.
pub fn validate_field_ownership(
    modules: &[(String, &dyn SimulationModule)],
) -> Result<(), OwnershipError> {
    let mut owners: BTreeMap<FieldKey, Vec<String>> = BTreeMap::new();
    for (id, module) in modules {
        for key in module.writes() {
            owners.entry(key).or_default().push(id.clone());
        }
    }
    for (field, writers) in owners {
        if writers.len() > 1 {
            return Err(OwnershipError::DuplicateWriter { field, writers });
        }
    }
    Ok(())
}
