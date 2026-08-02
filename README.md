# WorldSmith
<img width="1300" height="643" alt="Screenshot_116" src="https://github.com/user-attachments/assets/8512fc86-a191-43d2-9c42-a722cca8df63" />

*A deterministic planetary evolution engine written in Rust.*

WorldSmith is an open-source simulation engine for generating scientifically plausible planets through deterministic physical models. Instead of relying on procedural noise or arbitrary random generation, every planetary property emerges from reproducible simulations of geological, atmospheric, hydrological, and climatic processes.

Designed as reusable infrastructure rather than a game engine, WorldSmith provides developers, researchers, and simulation projects with a modular foundation for explainable procedural world generation.

---

## Why WorldSmith?

Most procedural planet generators focus on producing visually interesting worlds.

**WorldSmith focuses on producing explainable worlds.**

Every mountain range, atmosphere, ocean, and climate can be traced back to the physical processes that created it. Given the same seed and configuration, the engine always produces identical results, making simulations reproducible, debuggable, and scientifically inspectable.

The project is intended for:

- Scientific and educational simulations
- Procedural world generation
- Strategy and simulation games
- Space exploration projects
- Planetary visualization tools
- Research into deterministic simulation systems

---

<img width="729" height="647" alt="Screenshot_117" src="https://github.com/user-attachments/assets/cb299b2a-2a77-480a-a9e9-e1a3ece53190" />

## Features

- Deterministic simulation pipeline
- Modular physics architecture
- Plugin-based execution scheduler
- Directed Acyclic Graph (DAG) dependency system
- Reproducible fixed-seed simulations
- Planet snapshots
- Validation and invariant checking
- Replay testing
- Benchmarks
- Fully documented public API

---

## Simulation Pipeline

<img width="673" height="836" alt="Screenshot_118" src="https://github.com/user-attachments/assets/c33a77bb-0f46-417a-bdc9-2917122447a7" />

```text
Core
 ↓
Mantle
 ↓
Volcanism
 ↓
Plate Tectonics
 ↓
Atmosphere
 ↓
Hydrology
 ↓
Climate
 ↓
Carbon Cycle
 ↓
Biosphere
 ↓
Cryosphere
 ↓
Surface Chemistry
 ↓
Habitability
 ↓
Planet Classification
```

Each module represents an independent physical system with clearly defined inputs and outputs. Modules execute through a dependency-driven scheduler that guarantees deterministic execution order.

---

## Design Principles

### Deterministic

Identical seed + identical inputs = identical world.

WorldSmith guarantees bit-for-bit reproducible simulations.

### Explainable

Every piece of planetary state has a single owner.

No hidden mutations.

No unpredictable side effects.

Every result can be traced back to the module responsible for producing it.

### Modular

Simulation systems are isolated into independent modules.

New physics can be introduced without modifying existing systems.

### Renderer Independent

The engine contains no rendering, UI, or gameplay logic.

Visualization is intentionally separated from simulation.

---

## Architecture

<img width="847" height="830" alt="Screenshot_119" src="https://github.com/user-attachments/assets/30df4876-34c1-4105-a2f6-545991b36f85" />

| Crate | Purpose |
|--------|---------|
| `worldsmith-engine` | Scheduler and simulation engine |
| `worldsmith-evolution` | Physics simulation modules |
| `worldsmith-models` | Planet state definitions |
| `worldsmith-state` | Snapshots and world state |
| `worldsmith-traits` | Module interfaces |
| `worldsmith-stellar` | Orbital and stellar simulation |
| `worldsmith-validation` | Replay testing and validation |
| `worldsmith-rng` | Deterministic random generation |
| `worldsmith-units` | Physical constants |
| `worldsmith-grid` | Spatial indexing |

Additional crates provide visualization, serialization, UI, presets, and application tooling.

---

## Scientific Scope

### Currently Implemented

- Interior thermal evolution
- Core evolution
- Mantle dynamics
- Volcanism
- Plate tectonics
- Atmospheric evolution
- Hydrology
- Climate equilibrium
- Carbon cycle
- Biosphere simulation
- Cryosphere
- Surface chemistry
- Habitability assessment
- Planet classification

### Planned

- Ocean circulation
- Weather systems
- Cloud simulation
- Erosion
- Sediment transport
- Geological history
- Ecological evolution

---

## Determinism

WorldSmith is deterministic by design.

Reproducibility is guaranteed through:

- Fixed-seed RNG
- Pure simulation modules
- Deterministic scheduling
- Replay validation
- Snapshot verification
- No hidden mutable global state
- No runtime I/O during simulation

---

## Testing

<img width="900" height="807" alt="Screenshot_120" src="https://github.com/user-attachments/assets/ca4a8015-a4dc-4bd6-8154-13e491692aab" />

The project currently includes:

- **282+ automated tests**
- Unit tests
- Integration tests
- Deterministic replay tests
- Validation tests
- Snapshot verification
- Performance benchmarks

Automated validation ensures simulations remain physically consistent while preventing regressions as new systems are added.

---

## Benchmarks

| Operation | Performance |
|-----------|------------:|
| Planet generation | **1.58 µs** |
| 100 simulation ticks | **1.50 ms** |
| 1000 simulation ticks | **24.7 ms** |
| Snapshot creation | **1.16 µs** |

---

## Examples

Example programs demonstrate:

- Creating planets
- Running simulations
- Inspecting planetary state
- Snapshot generation
- Deterministic replay validation

```bash
cargo build --workspace

cargo test --workspace

cargo run --example create_planet -p worldsmith-engine
cargo run --example evolve_planet -p worldsmith-engine
cargo run --example inspect_planet -p worldsmith-engine
cargo run --example deterministic_replay -p worldsmith-engine
```

---

## Roadmap

### v1.0

- Deterministic simulation engine
- Physics modules
- Validation framework
- Replay testing
- Public API

### v2.0

- Snapshot serialization
- Interactive visualization
- Renderer
- Timeline playback
- Desktop application

### Future

- Weather simulation
- Ocean circulation
- Terrain generation
- Paleogeography
- Ecological evolution
- GPU acceleration

---

## Vision

WorldSmith aims to become an open-source foundation for deterministic planetary simulation.

Rather than generating planets that merely look believable, WorldSmith models the physical processes that create them. By combining scientific simulation with modern software architecture, it provides reusable infrastructure for anyone building educational software, scientific tools, simulation engines, or next-generation procedural worlds.

Whether you're creating a strategy game, researching planetary science, or building a procedural universe, WorldSmith is designed to provide deterministic, explainable, and extensible world simulation.

---

## License

Dual licensed under either of:

- MIT License
- Apache License 2.0

Choose whichever license best fits your project.
