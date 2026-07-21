//! Deterministic pipeline ordering and dependency validation.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use worldsmith_traits::PipelineStage;

use crate::error::{EngineError, EngineResult};

/// Description of a registered pipeline stage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PipelineStageDescriptor {
    /// Stable stage and module identifier.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Lower priority values execute earlier.
    pub priority: i32,
    /// Stage identifiers that must execute before this stage.
    pub dependencies: Vec<String>,
}

impl PipelineStageDescriptor {
    /// Creates a new stage descriptor.
    pub fn new(id: impl Into<String>, name: impl Into<String>, priority: i32) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            priority,
            dependencies: Vec::new(),
        }
    }

    /// Adds dependencies to this stage descriptor.
    pub fn with_dependencies(mut self, dependencies: Vec<String>) -> Self {
        self.dependencies = dependencies;
        self
    }
}

impl PipelineStage for PipelineStageDescriptor {
    fn stage_id(&self) -> &str {
        &self.id
    }

    fn stage_name(&self) -> &str {
        &self.name
    }

    fn order(&self) -> i32 {
        self.priority
    }

    fn dependencies(&self) -> Vec<String> {
        self.dependencies.clone()
    }
}

/// Deterministically ordered execution pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Pipeline {
    stages: Vec<PipelineStageDescriptor>,
    execution_order: Vec<String>,
}

impl Pipeline {
    /// Builds a validated pipeline from stage descriptors.
    pub fn build(stages: Vec<PipelineStageDescriptor>) -> EngineResult<Self> {
        let mut pipeline = Self {
            stages,
            execution_order: Vec::new(),
        };
        pipeline.validate()?;
        pipeline.execution_order = pipeline.resolve_order()?;
        Ok(pipeline)
    }

    /// Returns the ordered stage identifiers.
    pub fn execution_order(&self) -> &[String] {
        &self.execution_order
    }

    /// Returns all stage descriptors.
    pub fn stages(&self) -> &[PipelineStageDescriptor] {
        &self.stages
    }

    /// Returns a stage descriptor by identifier.
    pub fn stage(&self, id: &str) -> Option<&PipelineStageDescriptor> {
        self.stages.iter().find(|stage| stage.id == id)
    }

    /// Validates duplicate identifiers and missing dependencies.
    pub fn validate(&self) -> EngineResult<()> {
        let mut ids = BTreeSet::new();
        for stage in &self.stages {
            if !ids.insert(stage.id.clone()) {
                return Err(EngineError::DuplicateModule(stage.id.clone()));
            }
        }
        for stage in &self.stages {
            for dependency in &stage.dependencies {
                if !ids.contains(dependency) {
                    return Err(EngineError::MissingDependency {
                        module: stage.id.clone(),
                        dependency: dependency.clone(),
                    });
                }
            }
        }
        Ok(())
    }

    fn resolve_order(&self) -> EngineResult<Vec<String>> {
        let by_id: BTreeMap<String, PipelineStageDescriptor> = self
            .stages
            .iter()
            .cloned()
            .map(|stage| (stage.id.clone(), stage))
            .collect();
        let mut remaining: BTreeSet<String> = by_id.keys().cloned().collect();
        let mut resolved = BTreeSet::new();
        let mut order = Vec::new();

        while !remaining.is_empty() {
            let mut ready: Vec<_> = remaining
                .iter()
                .filter_map(|id| {
                    let stage = by_id.get(id)?;
                    if stage
                        .dependencies
                        .iter()
                        .all(|dependency| resolved.contains(dependency))
                    {
                        Some((stage.priority, stage.id.clone()))
                    } else {
                        None
                    }
                })
                .collect();

            if ready.is_empty() {
                return Err(EngineError::CircularDependency(
                    remaining.into_iter().collect::<Vec<_>>().join(", "),
                ));
            }

            ready.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
            let (_, id) = ready.remove(0);
            remaining.remove(&id);
            resolved.insert(id.clone());
            order.push(id);
        }

        Ok(order)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dependencies_override_priority() {
        let pipeline = Pipeline::build(vec![
            PipelineStageDescriptor::new("b", "B", 0).with_dependencies(vec!["a".to_string()]),
            PipelineStageDescriptor::new("a", "A", 10),
        ])
        .unwrap();
        assert_eq!(
            pipeline.execution_order(),
            vec!["a".to_string(), "b".to_string()]
        );
    }
}
