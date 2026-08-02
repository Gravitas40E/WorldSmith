<p align="center">

# WorldSmith

**An open-source deterministic planetary evolution engine written in Rust.**

![Rust](https://img.shields.io/badge/Built%20with-Rust-orange?logo=rust)
![License](https://img.shields.io/badge/License-Apache%202.0-blue)

</p>

<p align="center">
<img width="1300" alt="WorldSmith Hero" src="https://github.com/user-attachments/assets/8512fc86-a191-43d2-9c42-a722cca8df63">
</p>

WorldSmith is an open-source simulation engine for generating scientifically plausible planets through deterministic physical models.

Instead of relying on procedural noise or arbitrary randomness, every planetary property emerges from reproducible simulations of geological, atmospheric, hydrological, and climatic processes.

Designed as reusable infrastructure rather than a game engine, WorldSmith provides developers, researchers, and simulation projects with a modular foundation for explainable procedural world generation.

---

# Why WorldSmith?

Most procedural planet generators focus on creating worlds that *look* believable.

**WorldSmith focuses on creating worlds that are explainable.**

Every mountain range, atmosphere, ocean, and climate can be traced back to the physical processes that created it. Given the same seed and initial conditions, the engine always produces identical results, making simulations reproducible, debuggable, and scientifically inspectable.

WorldSmith is intended for:

- Scientific and educational simulations
- Procedural world generation
- Strategy and simulation games
- Space exploration projects
- Planetary visualization tools
- Research into deterministic simulation systems

<p align="center">
<img width="729" alt="WorldSmith Overview" src="https://github.com/user-attachments/assets/cb299b2a-2a77-480a-a9e9-e1a3ece53190">
</p>

---

# Features

- Deterministic simulation pipeline
- Modular physics architecture
- Plugin-based execution scheduler
- Directed Acyclic Graph (DAG) dependency system
- Reproducible fixed-seed simulations
- Immutable world snapshots
- Validation and invariant checking
- Deterministic replay testing
- Performance benchmarks
- Fully documented public API

---

# Simulation Pipeline

<p align="center">
<img width="673" alt="Simulation Pipeline" src="https://github.com/user-attachments/assets/c33a77bb-0f46-417a-bdc9-2917122447a7">
</p>

Each module represents an independent physical system with clearly defined inputs and outputs. Modules execute through a dependency-driven scheduler that guarantees deterministic execution while maintaining clear ownership of simulation state.

---

# Design Principles

## Deterministic

Identical seed + identical inputs = identical world.

WorldSmith guarantees bit-for-bit reproducible simulations.

## Explainable

Every piece of planetary state has a single owner.

No hidden mutations.

No unpredictable side effects.

Every result can be traced back to the module responsible for producing it.

## Modular

Simulation systems are isolated into independent modules.

New physics can be introduced without modifying existing systems.

## Renderer Independent

The simulation engine contains no rendering, UI, or gameplay logic.

Visualization is intentionally separated from simulation, allowing the engine to run headless or power multiple frontends.

---

# Why Rust?

WorldSmith is built in Rust because it provides the guarantees needed for long-running deterministic simulations.

- Memory safety without garbage collection
- High-performance native execution
- Strong type system for scientific simulation
- Excellent tooling and testing ecosystem
- Reliable concurrency without data races

---

# Architecture

<p align="center">
<img width="847" alt="Architecture" src="https://github.com/user-attachments/assets/30df4876-34c1-4105-a2f6-545991b36f85">
</p>

| Crate | Purpose |
|--------|---------|
| `worldsmith-engine` | Scheduler and simulation engine |
| `worldsmith-evolution` | Physics simulation modules |
| `worldsmith-models` | Planet state definitions |
| `worldsmith-state` | World state and snapshots |
| `worldsmith-traits` | Module interfaces |
| `worldsmith-stellar` | Stellar and orbital dynamics |
| `worldsmith-validation` | Replay testing and validation |
| `worldsmith-rng` | Deterministic random number generation |
| `worldsmith-units` | Physical units and constants |
| `worldsmith-grid` | Spatial indexing |

Additional crates provide visualization, serialization, UI, presets, and application tooling.

---

# Current Simulation Systems

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
- Erosion and sediment transport
- Geological history
- Ecological evolution

---

# Determinism

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

# Testing

<p align="center">
<img width="900" alt="Testing" src="https://github.com/user-attachments/assets/ca4a8015-a4dc-4bd6-8154-13e491692aab">
</p>

WorldSmith currently includes:

- **282+ automated tests**
- Unit tests
- Integration tests
- Deterministic replay tests
- Validation tests
- Snapshot verification
- Performance benchmarks

Automated validation ensures simulations remain physically consistent while preventing regressions as new systems are introduced.

---

# Benchmarks

> Current benchmark results measured on the existing simulation implementation.

| Operation | Performance |
|-----------|------------:|
| Planet generation | **1.58 µs** |
| 100 simulation ticks | **1.50 ms** |
| 1000 simulation ticks | **24.7 ms** |
| Snapshot creation | **1.16 µs** |

---

# Examples

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

# Roadmap

## v1.0

- Deterministic simulation engine
- Core physics modules
- Validation framework
- Replay testing
- Stable public API

## v2.0

- Snapshot serialization
- Interactive renderer
- Planet visualization
- Timeline playback
- Desktop application

## Future Research

- Weather simulation
- Ocean circulation
- Terrain generation
- Paleogeography
- Ecological evolution
- GPU acceleration

---

# Vision

WorldSmith aims to become an open-source foundation for deterministic planetary simulation.

Rather than generating planets that merely appear believable, WorldSmith models the physical processes that shape them. Every observable property can be traced back to deterministic simulation, making worlds reproducible, explainable, and scientifically inspectable.

Whether you're building a strategy game, researching planetary science, or creating a procedural universe, WorldSmith provides a modular foundation for deterministic world simulation.

---

# License

This project is licensed under the **Apache License 2.0**.

See the [LICENSE](LICENSE) file for details.
