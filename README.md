# WorldSmith

A deterministic planetary evolution engine written in Rust.

WorldSmith simulates the physical evolution of planets from initial conditions through deterministic, explainable scientific models. Every observable property—orbit, composition, interior, surface, atmosphere, climate, and habitability—emerges from simulated physical processes, not arbitrary procedural generation.

WorldSmith is **not** a procedural noise generator, game engine, or renderer. It is a scientific simulation engine that evolves planets through a fixed, reproducible pipeline.

## Overview

WorldSmith treats a planet as a set of coupled physical systems. Each system is modeled as an independent module that reads a defined set of inputs and writes exactly one state. The scheduler runs all modules in a deterministic order, producing bit-for-bit reproducible results for a given seed and initial conditions.

This design makes it possible to:

- Reproduce any world exactly from its seed
- Trace every planetary property back to the module that produced it
- Add new physics without changing existing modules
- Validate scientific output against known constraints

## Design Philosophy

- **Determinism** — Fixed seed, same inputs, same output every time.
- **Explainability** — Every state field has a single owner module.
- **Modular architecture** — Physics systems are independent and composable.
- **Single ownership** — One module writes each state; no other module mutates it.
- **Plugin architecture** — Modules register by priority and dependency; the scheduler builds a DAG at runtime.
- **Renderer independence** — Simulation has no coupling to graphics, UI, or gameplay.
- **Reproducibility** — Automated replay tests verify that identical seeds produce identical worlds.

## Status

**v1.0.0 released.** Core simulation modules implemented and tested. Public API is stable.

## Architecture

| Crate | Role |
|---|---|
| `worldsmith-math` | Vector types and math utilities |
| `worldsmith-models` | Planetary state structs, enums, properties |
| `worldsmith-state` | World state, events, snapshots, serialization descriptors |
| `worldsmith-traits` | `SimulationModule` trait, `StateWriter` |
| `worldsmith-engine` | Engine, scheduler, plugin pipeline, `EngineBuilder` |
| `worldsmith-evolution` | Physics simulation modules |
| `worldsmith-planet` | Planet construction and configuration |
| `worldsmith-stellar` | Stellar and orbital dynamics |
| `worldsmith-validation` | Invariant checks, replay tests, performance benchmarks |
| `worldsmith-rng` | Reproducible random number generation |
| `worldsmith-units` | Physical unit constants |
| `worldsmith-grid` | Spatial indexing |
| `worldsmith-io` | File I/O (planned) |
| `worldsmith-serialization` | Snapshot serialization (planned) |
| `worldsmith-visualization` | Data visualization (planned) |
| `worldsmith-ui` | User interface (planned) |
| `worldsmith-cli` | Command-line interface (planned) |
| `worldsmith-app` | Desktop application (planned) |
| `worldsmith-presets` | Preset configurations (planned) |

## Simulation Pipeline

```
Core → Mantle → Volcanism → Plate Tectonics → Atmosphere
  → Hydrology → Climate → Carbon Cycle → Biosphere
  → Cryosphere → Surface Chemistry → Habitability
  → Planet Classification
```

The pipeline is a Directed Acyclic Graph (DAG). Each arrow represents a data dependency. Modules at the same priority level run in registration order.

### Ownership

Every persistent state struct has exactly one owner module. No other module reads or writes that struct's mutable state.

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

## Scientific Scope

### Implemented in v1.0

- Interior thermal evolution
- Volcanism and magma generation
- Plate tectonics and crustal recycling
- Atmospheric composition and pressure
- Hydrology and water reservoirs
- Climate equilibrium and greenhouse effect
- Carbon cycle and carbonate-silicate weathering
- Biosphere and biomass dynamics
- Cryosphere and ice reservoirs
- Surface chemistry and weathering
- Habitability assessment
- Planet classification

### Not Included in v1.0

The following are intentionally excluded and reserved for future research or Phases beyond v1.0:

- Weather systems
- Rivers and ocean circulation
- Erosion and sediment transport
- Civilization or biosphere intelligence simulation
- Rendering, visualization, or gameplay systems
- Real-time interactive editing

## Testing

- **282 passing tests** covering unit, integration, validation, ownership, snapshot, and deterministic replay cases
- Automated tests verify that identical seeds produce identical snapshots
- Validation layer enforces physical bounds, NaN/Inf checks, and cross-module invariants

## Determinism

WorldSmith is deterministic by design. The same seed, planet configuration, and tick count always produce the same simulation state. This property is verified by automated replay tests and is guaranteed by:

- Pure-function modules with no hidden state
- Fixed-seed RNG consumed in a reproducible order
- No I/O or timers during `tick()`

## Benchmarks

Measured on `worldsmith-engine` with stubbed modules in release profile:

| Benchmark | Result |
|---|---|
| Planet generation | 1.58 µs |
| 100 ticks | 1.50 ms |
| 1000 ticks | 24.7 ms |
| Snapshot creation | 1.16 µs |

## Examples

Five example programs are included with `worldsmith-engine`:

| Example | Description |
|---|---|
| `create_planet.rs` | Create a planet and initialize it in the engine |
| `evolve_planet.rs` | Evolve a planet through 100 ticks |
| `save_snapshot.rs` | Capture a simulation snapshot |
| `deterministic_replay.rs` | Verify identical output from two engines with the same seed |
| `inspect_planet.rs` | Print atmosphere, hydro, climate, and classification state |

## Getting Started

```bash
# Build everything
cargo build --workspace

# Run tests
cargo test --workspace

# Run examples
cargo run --example create_planet -p worldsmith-engine
cargo run --example evolve_planet -p worldsmith-engine
cargo run --example inspect_planet -p worldsmith-engine
cargo run --example deterministic_replay -p worldsmith-engine
```

## Project Structure

```
crates/
  worldsmith-math/        Linear algebra and utilities
  worldsmith-models/      Planetary types and state definitions
  worldsmith-state/       World state, events, snapshots, field registry
  worldsmith-traits/      SimulationModule trait and consumers
  worldsmith-evolution/   Core simulation modules (physics pipeline)
  worldsmith-engine/      Engine, scheduler, pipeline, builder
  worldsmith-planet/      Planet construction utilities
  worldsmith-validation/  Invariants, replay, performance, cross-module tests
  worldsmith-stellar/     Stellar and orbital dynamics
  worldsmith-rng/         Reproducible RNG utilities
  worldsmith-units/       Unit constants and conversions
  worldsmith-grid/        Spatial indexing
  worldsmith-io/          File I/O (planned)
  worldsmith-serialization/ Snapshot serialization (planned)
  ...
examples/                 Runnable engine examples
docs/                     Architecture, getting started, simulation overview
```

## Key Concepts

- **Deterministic**: Fixed seed → identical simulation every time.
- **Single ownership**: One module writes each state; no other module mutates it.
- **Snapshots**: Immutable captures of the full world state at any tick.
- **Plugins**: Modules register by priority and dependency; the scheduler runs a DAG.

## Roadmap

### v1.0 — Science Core (Complete)

- 13 evolution modules
- Deterministic scheduler
- Snapshot system
- Validation layer
- Public API
- Documentation
- Benchmarks
- Examples

### v2.0 — Renderer and I/O

- OpenGL / WebGPU renderer
- Snapshot save/load
- Interactive timeline scrubbing
- Terrain and atmosphere visualization

### Future Research

- Oceans and ocean circulation
- Weather and cloud systems
- Erosion and sediment transport
- Plate motion history and paleogeography
- Life evolution and ecological dynamics

## Vision

WorldSmith exists to make procedural worlds explainable. Every mountain, ocean, and atmosphere should be traceable to the physical processes that created it. By combining deterministic simulation with rigorous modular architecture, WorldSmith provides a foundation for scientific worldbuilding that is reproducible, extensible, and rooted in real physics.

## License

MIT OR Apache-2.0
