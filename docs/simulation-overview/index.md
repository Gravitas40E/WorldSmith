# Simulation Overview – WorldSmith Engine v1.0

## Pipeline

WorldSmith runs a deterministic, single-pass simulation pipeline. Each tick calls `Engine::tick(delta_seconds)`, which executes every registered module in priority order. Modules that share the same priority run in registration order.

```
Core → Mantle → Volcanism → Plate Tectonics → Atmosphere
  → Hydrology → Climate → Carbon Cycle → Biosphere
  → Cryosphere → Surface Chemistry → Habitability → Planet Classification
```

## Module Ownership

Every simulation state struct has a single owner module. No other module reads or writes that struct's mutable state. This eliminates race conditions and makes reproducibility trivial.

| State Struct | Owner Module |
|---|---|
| `CoreState` | `CoreEvolutionModule` |
| `MantleState` | `MantleEvolutionModule` |
| `VolcanismState` | `VolcanismModule` |
| `PlateTectonicsState` | `PlateTectonicsModule` |
| `AtmosphereState` | `AtmosphereModule` |
| `HydrologyState` | `HydrologyModule` |
| `ClimateState` | `ClimateModule` |
| `CarbonCycleState` | `CarbonCycleModule` |
| `BiosphereState` | `BiosphereModule` |
| `CryosphereState` | `CryosphereModule` |
| `SurfaceChemistryState` | `SurfaceChemistryModule` |
| `HabitabilityState` | `HabitabilityModule` |
| `PlanetClassificationState` | `PlanetClassificationModule` |

Assessment modules (`Habitability`, `PlanetClassification`) do not influence any physical state.

## Plugin Architecture

Modules implement `SimulationModule` and register with `EngineBuilder`. Each registration supplies:

```rust
.register_module_with_stage(
    Box::new(MyModule::default()),
    priority: i32,
    dependencies: Vec<String>,
)
```

The scheduler builds a DAG from priorities and dependencies. Cycles are rejected at build time.

## Determinism

- Fixed seed: `EngineBuilder::with_seed(N)`
- State mutations are pure functions of `(seed, state, delta_seconds)`
- Observers read snapshots; no hidden timers or I/O during `tick()`

## Snapshot System

Snapshots capture:

- `SimulationMetadata` — ID, timestamps, schema version
- `timestamp_s` — simulation clock
- `StellarSnapshot` — stars
- `PlanetSnapshot` — full state of every planet
- `MoonSnapshot` — full state of every moon

Snapshots are immutable once captured.
