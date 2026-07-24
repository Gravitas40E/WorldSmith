//! Convenience registration for planet evolution modules.
//!
//! [`EvolutionPlugin`] bundles the Phase 10A placeholder modules
//! (`CoreEvolutionModule` and `MantleEvolutionModule`) and exposes a single
//! entry point for engine registration.  It is intentionally thin: the engine
//! already owns module ordering and pipeline validation, so this type merely
//! avoids boilerplate in examples and tests.
//!
//! ## Usage
//!
//! ```ignore
//! use worldsmith_evolution::EvolutionPlugin;
//! use worldsmith_engine::EngineBuilder;
//!
//! let builder = EvolutionPlugin::new().register_with(EngineBuilder::new());
//! let engine = builder.build().unwrap();
//! ```

use worldsmith_engine::EngineBuilder;
use worldsmith_engine::PipelineStageDescriptor;

use crate::{
    AtmosphereModule, BiosphereModule, CarbonCycleModule, ClimateModule, CoreEvolutionModule,
    CryosphereModule, HabitabilityModule, HydrologyModule, MantleEvolutionModule,
    PlanetClassificationModule, PlateTectonicsModule, SurfaceChemistryModule, VolcanismModule,
};

/// Default pipeline priority values for Phase 10A modules.
///
/// Lower priorities execute earlier.  Core must run before mantle so that
/// mantle can depend on core heat-flux outputs in future phases.
pub struct EvolutionPriorities;

impl EvolutionPriorities {
    /// Priority assigned to `worldsmith.evolution.core`.
    pub const CORE: i32 = 100;
    /// Priority assigned to `worldsmith.evolution.mantle`.
    pub const MANTLE: i32 = 50;
    /// Priority assigned to `worldsmith.evolution.volcanism`.
    pub const VOLCANISM: i32 = 40;
    /// Priority assigned to `worldsmith.evolution.plate_tectonics`.
    pub const PLATE_TECTONICS: i32 = 30;
    /// Priority assigned to `worldsmith.evolution.atmosphere`.
    pub const ATMOSPHERE: i32 = 25;
    /// Priority assigned to `worldsmith.evolution.hydrology`.
    pub const HYDROLOGY: i32 = 20;
    /// Priority assigned to `worldsmith.evolution.climate`.
    pub const CLIMATE: i32 = 15;
    /// Priority assigned to `worldsmith.evolution.carbon_cycle`.
    pub const CARBON_CYCLE: i32 = 10;
    /// Priority assigned to `worldsmith.evolution.biosphere`.
    pub const BIOSPHERE: i32 = 5;
    /// Priority assigned to `worldsmith.evolution.cryosphere`.
    pub const CRYOSPHERE: i32 = 2;
    /// Priority assigned to `worldsmith.evolution.surface_chemistry`.
    pub const SURFACE_CHEMISTRY: i32 = 1;
    /// Priority assigned to `worldsmith.evolution.habitability`.
    pub const HABITABILITY: i32 = 0;
    /// Priority assigned to `worldsmith.evolution.planet_classification`.
    pub const PLANET_CLASSIFICATION: i32 = -1;
}

/// Convenience wrapper that registers the Phase 10A evolution modules.
#[derive(Debug, Default)]
pub struct EvolutionPlugin {
    /// Whether to register the core module.
    pub core: bool,
    /// Whether to register the mantle module.
    pub mantle: bool,
    /// Whether to register the volcanism module.
    pub volcanism: bool,
    /// Whether to register the plate tectonics module.
    pub plate_tectonics: bool,
    /// Whether to register the atmosphere module.
    pub atmosphere: bool,
    /// Whether to register the hydrology module.
    pub hydrology: bool,
    /// Whether to register the climate module.
    pub climate: bool,
    /// Whether to register the carbon cycle module.
    pub carbon_cycle: bool,
    /// Whether to register the biosphere module.
    pub biosphere: bool,
    /// Whether to register the cryosphere module.
    pub cryosphere: bool,
    /// Whether to register the surface chemistry module.
    pub surface_chemistry: bool,
    /// Whether to register the habitability module.
    pub habitability: bool,
    /// Whether to register the planet classification module.
    pub planet_classification: bool,
}

impl EvolutionPlugin {
    /// Creates a plugin that registers all Phase 10A placeholder modules.
    pub fn new() -> Self {
        Self {
            core: true,
            mantle: true,
            volcanism: true,
            plate_tectonics: true,
            atmosphere: true,
            hydrology: true,
            climate: true,
            carbon_cycle: true,
            biosphere: true,
            cryosphere: true,
            surface_chemistry: true,
            habitability: true,
            planet_classification: true,
        }
    }

    /// Registers selected evolution modules into an [`EngineBuilder`].
    pub fn register_with(self, builder: EngineBuilder) -> EngineBuilder {
        let mut next = builder;
        if self.core {
            next = next.register_module_with_stage(
                Box::new(CoreEvolutionModule::default()),
                EvolutionPriorities::CORE,
                Vec::new(),
            );
        }
        if self.mantle {
            next = next.register_module_with_stage(
                Box::new(MantleEvolutionModule::default()),
                EvolutionPriorities::MANTLE,
                vec!["worldsmith.evolution.core".to_string()],
            );
        }
        if self.volcanism {
            next = next.register_module_with_stage(
                Box::new(VolcanismModule::default()),
                EvolutionPriorities::VOLCANISM,
                vec!["worldsmith.evolution.mantle".to_string()],
            );
        }
        if self.plate_tectonics {
            next = next.register_module_with_stage(
                Box::new(PlateTectonicsModule::default()),
                EvolutionPriorities::PLATE_TECTONICS,
                vec!["worldsmith.evolution.volcanism".to_string()],
            );
        }
        if self.atmosphere {
            next = next.register_module_with_stage(
                Box::new(AtmosphereModule::default()),
                EvolutionPriorities::ATMOSPHERE,
                vec!["worldsmith.evolution.plate_tectonics".to_string()],
            );
        }
        if self.hydrology {
            next = next.register_module_with_stage(
                Box::new(HydrologyModule::default()),
                EvolutionPriorities::HYDROLOGY,
                vec!["worldsmith.evolution.atmosphere".to_string()],
            );
        }
        if self.climate {
            next = next.register_module_with_stage(
                Box::new(ClimateModule::default()),
                EvolutionPriorities::CLIMATE,
                vec!["worldsmith.evolution.hydrology".to_string()],
            );
        }
        if self.carbon_cycle {
            next = next.register_module_with_stage(
                Box::new(CarbonCycleModule::default()),
                EvolutionPriorities::CARBON_CYCLE,
                vec!["worldsmith.evolution.climate".to_string()],
            );
        }
        if self.biosphere {
            next = next.register_module_with_stage(
                Box::new(BiosphereModule::default()),
                EvolutionPriorities::BIOSPHERE,
                vec!["worldsmith.evolution.carbon_cycle".to_string()],
            );
        }
        if self.cryosphere {
            next = next.register_module_with_stage(
                Box::new(CryosphereModule::default()),
                EvolutionPriorities::CRYOSPHERE,
                vec!["worldsmith.evolution.biosphere".to_string()],
            );
        }
        if self.surface_chemistry {
            next = next.register_module_with_stage(
                Box::new(SurfaceChemistryModule::default()),
                EvolutionPriorities::SURFACE_CHEMISTRY,
                vec!["worldsmith.evolution.cryosphere".to_string()],
            );
        }
        if self.habitability {
            next = next.register_module_with_stage(
                Box::new(HabitabilityModule::default()),
                EvolutionPriorities::HABITABILITY,
                vec!["worldsmith.evolution.surface_chemistry".to_string()],
            );
        }
        if self.planet_classification {
            next = next.register_module_with_stage(
                Box::new(PlanetClassificationModule::default()),
                EvolutionPriorities::PLANET_CLASSIFICATION,
                vec!["worldsmith.evolution.habitability".to_string()],
            );
        }
        next
    }

    /// Returns the registered module descriptors for the enabled modules.
    pub fn descriptors(&self) -> Vec<PipelineStageDescriptor> {
        let mut descriptors = Vec::new();
        if self.core {
            descriptors.push(PipelineStageDescriptor::new(
                "worldsmith.evolution.core",
                "Planet Core Evolution",
                EvolutionPriorities::CORE,
            ));
        }
        if self.mantle {
            descriptors.push(PipelineStageDescriptor::new(
                "worldsmith.evolution.mantle",
                "Planet Mantle Evolution",
                EvolutionPriorities::MANTLE,
            ));
        }
        if self.volcanism {
            descriptors.push(PipelineStageDescriptor::new(
                "worldsmith.evolution.volcanism",
                "Planet Volcanism Evolution",
                EvolutionPriorities::VOLCANISM,
            ));
        }
        if self.plate_tectonics {
            descriptors.push(PipelineStageDescriptor::new(
                "worldsmith.evolution.plate_tectonics",
                "Planet Plate Tectonics Evolution",
                EvolutionPriorities::PLATE_TECTONICS,
            ));
        }
        if self.atmosphere {
            descriptors.push(PipelineStageDescriptor::new(
                "worldsmith.evolution.atmosphere",
                "Planet Atmosphere Evolution",
                EvolutionPriorities::ATMOSPHERE,
            ));
        }
        if self.hydrology {
            descriptors.push(PipelineStageDescriptor::new(
                "worldsmith.evolution.hydrology",
                "Planet Hydrology Evolution",
                EvolutionPriorities::HYDROLOGY,
            ));
        }
        if self.climate {
            descriptors.push(PipelineStageDescriptor::new(
                "worldsmith.evolution.climate",
                "Planet Climate Evolution",
                EvolutionPriorities::CLIMATE,
            ));
        }
        if self.carbon_cycle {
            descriptors.push(PipelineStageDescriptor::new(
                "worldsmith.evolution.carbon_cycle",
                "Planet Carbon Cycle Evolution",
                EvolutionPriorities::CARBON_CYCLE,
            ));
        }
        if self.biosphere {
            descriptors.push(PipelineStageDescriptor::new(
                "worldsmith.evolution.biosphere",
                "Planet Biosphere Evolution",
                EvolutionPriorities::BIOSPHERE,
            ));
        }
        if self.cryosphere {
            descriptors.push(PipelineStageDescriptor::new(
                "worldsmith.evolution.cryosphere",
                "Planet Cryosphere Evolution",
                EvolutionPriorities::CRYOSPHERE,
            ));
        }
        if self.surface_chemistry {
            descriptors.push(PipelineStageDescriptor::new(
                "worldsmith.evolution.surface_chemistry",
                "Planet Surface Chemistry Evolution",
                EvolutionPriorities::SURFACE_CHEMISTRY,
            ));
        }
        if self.habitability {
            descriptors.push(PipelineStageDescriptor::new(
                "worldsmith.evolution.habitability",
                "Planet Habitability Assessment",
                EvolutionPriorities::HABITABILITY,
            ));
        }
        if self.planet_classification {
            descriptors.push(PipelineStageDescriptor::new(
                "worldsmith.evolution.planet_classification",
                "Planet Classification",
                EvolutionPriorities::PLANET_CLASSIFICATION,
            ));
        }
        descriptors
    }
}
