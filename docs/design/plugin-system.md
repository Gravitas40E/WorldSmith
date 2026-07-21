# Plugin System

How future modules (biosphere, civilizations, binary stars, moons, oceans) integrate without rewriting the engine.

## Core Traits (worldsmith-traits)

### `PipelineStage`

For generation-time or one-shot transformations. Registered with the engine at startup.

### `SimulationModule`

For ongoing runtime simulation. Declares:

- `tick_rate()` — suggested interval in simulation seconds
- `reads()` / `writes()` — field registry declarations
- `subscribes()` — event types consumed
- `tick(&mut WorldState, &StageContext)` — advancement

### `VisualLayer`

Maps a named field or derived quantity to a renderable representation. Registered with visualization crate.

### `ExportFormat`

Serializes a subset of `WorldState` to an external format.

## Registration

```text
EngineBuilder::new()
    .register_stage(stellar::FormationStage)
    .register_module(climate::ClimateModule)
    .register_visual_layer(climate::TemperatureLayer)
    .register_export(export::VtkFormat)
    .build()
```

No dynamic loading in Phase 1–3 (static linking). Dynamic plugins via `dlopen` are a future option once ABI stabilizes.

## Dependency Rules

```
Allowed:
  worldsmith-biosphere → models, traits, chemistry, climate

Forbidden:
  worldsmith-biosphere → render, ui, app
  worldsmith-geology → climate (use events/fields instead)
```

## Adding Binary Stars (Example)

1. Extend `StellarSystem` in `worldsmith-models` with `bodies: Vec<StarBody>`.
2. Add N-body orbital submodule in `worldsmith-stellar`.
3. Update `climate` insolation calculator to sum flux from all bodies.
4. No changes to `worldsmith-render` beyond new snapshot metadata.

## Adding Biosphere (Example)

1. New crate `worldsmith-biosphere`.
2. Subscribes to `SurfaceTemperatureChanged`, `AtmosphereCompositionChanged`.
3. Writes fields: `biomass_density`, `surface_metabolism_flux`.
4. Publishes `BiogenicGasFluxChanged` for atmosphere/chemistry.
5. Optional `VisualLayer` for biome map.

The engine requires no modification beyond registration.

## Chemistry as Data-Driven Plugin

Reaction networks live in `assets/chemistry/*.ron`. The chemistry engine interprets them at runtime—new networks without recompilation.
