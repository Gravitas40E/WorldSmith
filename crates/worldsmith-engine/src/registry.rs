//! Module registry that owns simulation modules.

use std::collections::BTreeMap;

use worldsmith_traits::{SimulationModule, StateWriter};

use crate::{
    error::{EngineError, EngineResult},
    pipeline::PipelineStageDescriptor,
};

/// Owned module plus its deterministic pipeline metadata.
pub struct RegisteredModule {
    /// Pipeline stage descriptor for this module.
    pub descriptor: PipelineStageDescriptor,
    module: Box<dyn SimulationModule>,
}

impl RegisteredModule {
    /// Creates a registered module.
    pub fn new(
        module: Box<dyn SimulationModule>,
        priority: i32,
        dependencies: Vec<String>,
    ) -> Self {
        let id = module.id().to_string();
        let name = module.name().to_string();
        Self {
            descriptor: PipelineStageDescriptor::new(id, name, priority)
                .with_dependencies(dependencies),
            module,
        }
    }

    /// Returns an immutable module reference.
    pub fn module(&self) -> &dyn SimulationModule {
        self.module.as_ref()
    }

    /// Returns a mutable module reference.
    pub fn module_mut(&mut self) -> &mut dyn SimulationModule {
        self.module.as_mut()
    }
}

/// Deterministic registry for all simulation modules.
#[derive(Default)]
pub struct ModuleRegistry {
    modules: BTreeMap<String, RegisteredModule>,
}

impl ModuleRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self {
            modules: BTreeMap::new(),
        }
    }

    /// Registers a module and validates duplicate identifiers.
    pub fn register(&mut self, module: RegisteredModule) -> EngineResult<()> {
        let id = module.descriptor.id.clone();
        if self.modules.contains_key(&id) {
            return Err(EngineError::DuplicateModule(id));
        }
        self.modules.insert(id, module);
        Ok(())
    }

    /// Returns whether a module id is registered.
    pub fn contains(&self, id: &str) -> bool {
        self.modules.contains_key(id)
    }

    /// Looks up a registered module.
    pub fn get(&self, id: &str) -> Option<&RegisteredModule> {
        self.modules.get(id)
    }

    /// Looks up a registered module mutably.
    pub fn get_mut(&mut self, id: &str) -> Option<&mut RegisteredModule> {
        self.modules.get_mut(id)
    }

    /// Returns module descriptors in deterministic identifier order.
    pub fn descriptors(&self) -> Vec<PipelineStageDescriptor> {
        self.modules
            .values()
            .map(|module| module.descriptor.clone())
            .collect()
    }

    /// Returns registered module identifiers in deterministic order.
    pub fn ids(&self) -> Vec<String> {
        self.modules.keys().cloned().collect()
    }

    /// Initializes modules in provided pipeline order.
    pub fn initialize(
        &mut self,
        order: &[String],
        state: &mut dyn StateWriter,
    ) -> EngineResult<()> {
        for id in order {
            let module =
                self.modules
                    .get_mut(id)
                    .ok_or_else(|| EngineError::MissingDependency {
                        module: id.clone(),
                        dependency: id.clone(),
                    })?;
            module.module_mut().initialize(state)?;
        }
        Ok(())
    }

    /// Shuts down modules in reverse pipeline order.
    pub fn shutdown(&mut self, order: &[String], state: &mut dyn StateWriter) -> EngineResult<()> {
        for id in order.iter().rev() {
            if let Some(module) = self.modules.get_mut(id) {
                module.module_mut().shutdown(state)?;
            }
        }
        Ok(())
    }
}
