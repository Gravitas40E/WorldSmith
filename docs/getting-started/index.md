# Getting Started – WorldSmith Engine v1.0

## Prerequisites

- Rust 1.75 or later
- Cargo (bundled with Rust)

## Build

```bash
git clone https://github.com/worldsmith/worldsmith.git
cd worldsmith
cargo build --workspace
```

## Run Tests

```bash
cargo test --workspace
```

## Run Examples

```bash
# Create and initialize a planet
cargo run --example create_planet -p worldsmith-engine

# Evolve a planet for 100 ticks
cargo run --example evolve_planet -p worldsmith-engine

# Inspect planet state after evolution
cargo run --example inspect_planet -p worldsmith-engine

# Verify deterministic replay
cargo run --example deterministic_replay -p worldsmith-engine

# Capture and inspect a snapshot
cargo run --example save_snapshot -p worldsmith-engine
```

## Crate Layout

| Crate | Purpose |
|---|---|
| `worldsmith-math` | Vector types, math utilities |
| `worldsmith-models` | State structs, enums, planetary properties |
| `worldsmith-state` | World state, events, snapshots, field descriptors |
| `worldsmith-traits` | `SimulationModule` trait, `StateWriter` |
| `worldsmith-engine` | Engine, scheduler, pipeline, `EngineBuilder` |
| `worldsmith-evolution` | Physics simulation modules |
| `worldsmith-planet` | Planet construction and configuration |
| `worldsmith-validation` | Invariants, replay tests, performance benchmarks |
| `worldsmith-stellar` | Stellar and orbital dynamics |
| `worldsmith-serialization` | Serialization (planned) |
| `worldsmith-io` | File I/O (planned) |
| `worldsmith-render` | Rendering (planned) |
| `worldsmith-visualization` | Data visualization (planned) |
| `worldsmith-ui` | User interface (planned) |
| `worldsmith-cli` | Command-line interface (planned) |
| `worldsmith-app` | Application entry point (planned) |
| `worldsmith-presets` | Preset configs (planned) |
| `worldsmith-chemistry` | Chemistry (planned) |
| `worldsmith-geology` | Geology (planned) |
| `worldsmith-atmosphere` | Atmosphere (planned) |
| `worldsmith-climate` | Climate (planned) |
| `worldsmith-grid` | Spatial grid (planned) |
| `worldsmith-rng` | Random number generation |
| `worldsmith-units` | Unit constants and conversions |
| `worldsmith-export` | Export (planned) |

## First Simulation

```rust
use worldsmith_engine::EngineBuilder;
use worldsmith_models::{Planet, PlanetId};

fn main() -> worldsmith_engine::EngineResult<()> {
    let mut engine = EngineBuilder::new()
        .with_seed(42)
        .build()?;

    engine.state_mut().planets.insert(PlanetId(1), simple_planet());
    engine.initialize()?;
    engine.tick(100.0)?;
    Ok(())
}
```

## Determinism

Always set a seed with `.with_seed(N)`. The same seed, same planetary inputs, and same tick count will always produce identical snapshots.

## Documentation

- `docs/simulation-overview/` — pipeline, modules, ownership, snapshots
- `docs/architecture/` — ADRs and audits
- API docs: `cargo doc --open`
