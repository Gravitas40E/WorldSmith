# WorldSmith Architecture

> Phase 1 — Foundation design. No simulation implementation yet.

WorldSmith is a **deterministic, physics-first planetary simulation engine**. Every observable property of a generated world must trace back to initial conditions and simulated processes—not procedural noise dressed as science.

This document defines the software architecture: module boundaries, communication patterns, data flow, and scalability path.

---

## Table of Contents

1. [Design Principles](#design-principles)
2. [Technology Stack](#technology-stack)
3. [High-Level Architecture](#high-level-architecture)
4. [Crate Map & Responsibilities](#crate-map--responsibilities)
5. [Simulation Architecture](#simulation-architecture)
6. [Planet Generation Pipeline](#planet-generation-pipeline)
7. [Data Flow & State Management](#data-flow--state-management)
8. [Module Communication](#module-communication)
9. [Rendering & Visualization Separation](#rendering--visualization-separation)
10. [Plugin & Extension Model](#plugin--extension-model)
11. [Naming Conventions](#naming-conventions)
12. [File Responsibilities](#file-responsibilities)
13. [Future Scalability](#future-scalability)
14. [Folder Structure](#folder-structure)

---

## Design Principles

| Principle | Rationale |
|-----------|-----------|
| **Determinism** | Same seed + same version → identical world. Required for science, reproducibility, and save/load integrity. |
| **Physics-first** | Modules implement equations and conservation laws. No "artist knobs" in the simulation core. |
| **Separation of concerns** | Simulation, rendering, UI, and I/O are independent crates with explicit interfaces. |
| **Unidirectional data flow** | UI and render read snapshots; they never mutate simulation state directly. |
| **Multi-rate time** | Orbital, geological, climatic, and chemical processes run on appropriate timescales. |
| **Typed boundaries** | All cross-module communication uses explicit Rust types—no stringly-typed event payloads. |
| **Incremental build** | Each crate compiles and tests in isolation. The engine grows by adding pipeline stages, not rewriting core. |

---

## Technology Stack

### Core Simulation — Rust

| Choice | Why |
|--------|-----|
| **Rust** | Memory safety without GC pauses; deterministic execution; strong type system for physical units; excellent parallel performance via `rayon`. |
| **Cargo workspace** | Monorepo of focused crates. Clear dependency graph enforced by the compiler. |
| **Custom physics** | Domain-specific solvers (orbital mechanics, hydrostatic equilibrium, radiative transfer)—not a generic game-physics engine. |

### Math & Numerics

| Library / Crate | Role |
|-----------------|------|
| `worldsmith-math` | Vectors, spherical geometry, interpolation, numerical integration—engine-owned, no hidden globals. |
| `worldsmith-units` | Dimensional analysis (`Length`, `Mass`, `Temperature`) to prevent unit errors at compile time. |
| `nalgebra` (optional, internal) | Linear algebra for grid solvers where appropriate. |

### Deterministic Randomness

| Choice | Why |
|--------|-----|
| `worldsmith-rng` | ChaCha8-based streams, one sub-stream per module derived from master seed. Fully reproducible. |

### Grids & Spatial Data

| Choice | Why |
|--------|-----|
| `worldsmith-grid` | Spherical HEALPix / lat-lon grids, layer stacks, halos. Shared by geology, atmosphere, climate. |

### Serialization & Persistence

| Format | Use |
|--------|-----|
| **RON** | Human-readable configs and presets. |
| **bincode** (versioned) | Binary save files—fast, compact, schema-versioned. |
| **JSON Schema** | Export interchange for external tools. |

### Rendering Engine

| Choice | Why |
|--------|-----|
| **wgpu** | Cross-platform (Vulkan/Metal/DX12/WebGPU); decoupled from simulation; GPU compute for field visualization. |
| **winit** | Windowing and input for the desktop app. |

Rendering lives in `worldsmith-render`. It consumes **visual snapshots** only—it never imports simulation solvers.

### UI

| Choice | Why |
|--------|-----|
| **egui** | Immediate-mode UI integrated with wgpu; ideal for scientific parameter panels, timelines, and inspectors. |

UI lives in `worldsmith-ui`. It sends **commands** to the engine and displays **read-only state**.

### CLI & Headless

| Choice | Why |
|--------|-----|
| **clap** | Command-line interface for batch generation, benchmarking, and CI. |
| `worldsmith-cli` | Runs the full pipeline without opening a window. |

### Testing & Benchmarks

| Tool | Role |
|------|------|
| `cargo test` | Unit and integration tests per crate. |
| `criterion` | Performance regression benchmarks for hot paths. |
| Golden-file tests | Determinism verification: fixed seed → fixed output hash. |

---

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           APPLICATION LAYER                             │
│  worldsmith-app (desktop)          worldsmith-cli (headless)            │
└───────────────────────────────┬─────────────────────────────────────────┘
                                │ commands / config
                                ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                         ORCHESTRATION LAYER                             │
│  worldsmith-engine          worldsmith-state          worldsmith-presets│
│  (pipeline, scheduler)      (authoritative world)     (initial conditions)│
└───────────────────────────────┬─────────────────────────────────────────┘
                                │ stage I/O
                                ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                      DOMAIN SIMULATION LAYER                            │
│  stellar │ planet │ geology │ atmosphere │ climate │ chemistry          │
└───────────────────────────────┬─────────────────────────────────────────┘
                                │ reads/writes typed fields
                                ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                           DATA LAYER                                    │
│  worldsmith-models          worldsmith-serialization                    │
└───────────────────────────────┬─────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                         FOUNDATION LAYER                                │
│  math │ units │ rng │ grid │ traits                                     │
└─────────────────────────────────────────────────────────────────────────┘

        ┌──────────────── PRESENTATION (read-only) ────────────────┐
        │  visualization → render → ui                             │
        │  (snapshots in, pixels & panels out)                     │
        └──────────────────────────────────────────────────────────┘

        ┌──────────────── I/O (side channel) ──────────────────────┐
        │  io (save/load)          export (VTK, GeoTIFF, JSON)     │
        └──────────────────────────────────────────────────────────┘
```

**Key invariant:** Dependencies flow **downward**. Presentation and I/O crates depend on models and snapshots—they never depend on solvers.

---

## Crate Map & Responsibilities

### Foundation Layer

#### `worldsmith-math`
**Why it exists:** Shared numerical primitives used by every domain module.  
**Why separated:** Prevents duplicated vector/spherical math and keeps domain crates free of low-level numerics.  
**Responsibilities:** 3D vectors, spherical coordinates, interpolation, ODE integrators, root-finding.

#### `worldsmith-units`
**Why it exists:** Physical quantities must not be confused (meters vs. kilometers, Kelvin vs. Celsius).  
**Why separated:** Units are cross-cutting; embedding them in each domain crate causes inconsistency.  
**Responsibilities:** Newtype wrappers for SI dimensions, unit conversion, display formatting.

#### `worldsmith-rng`
**Why it exists:** Deterministic stochastic processes (accretion variance, micro-turbulence seeds).  
**Why separated:** Centralizes RNG policy; modules request named sub-streams instead of owning `rand` directly.  
**Responsibilities:** Master seed, derived sub-streams, stream splitting for parallel workers.

#### `worldsmith-grid`
**Why it exists:** Surface and atmospheric fields share grid topology requirements.  
**Why separated:** Grid logic is substantial and reused; geology and climate should not each implement spherical grids.  
**Responsibilities:** Grid definitions, indexing, neighbor lookup, halos, regridding between resolutions.

#### `worldsmith-traits`
**Why it exists:** Defines the contracts (traits) that modules and the engine agree on.  
**Why separated:** Breaks circular dependencies; domain crates implement traits, engine consumes them.  
**Responsibilities:** `SimulationModule`, `PipelineStage`, `VisualLayer`, `ExportFormat` trait definitions.

---

### Data Layer

#### `worldsmith-models`
**Why it exists:** Single source of truth for all structured simulation data.  
**Why separated:** Domain modules depend on models, not on each other. Models contain no logic.  
**Responsibilities:** `StellarSystem`, `PlanetBody`, `AtmosphereState`, `GeologyState`, `ClimateState`, field buffers, identifiers.

#### `worldsmith-presets`
**Why it exists:** Curated initial conditions (solar analog, hot Jupiter seed, Archean Earth).  
**Why separated:** Presets are data + validation, not simulation. Users and tests load presets without running the engine.  
**Responsibilities:** Preset schemas, loading, validation against physical bounds.

#### `worldsmith-serialization`
**Why it exists:** Versioned encode/decode for models and save files.  
**Why separated:** Serialization formats evolve; domain code should not carry `serde` version migration logic.  
**Responsibilities:** Schema versioning, migration, checksums, compression hooks.

---

### Domain Simulation Layer

Each domain crate is **self-contained**: it reads specific model slices, advances state by Δt (or pipeline phase), writes results back.

#### `worldsmith-stellar`
**Why separated:** Stellar properties (mass, luminosity, spectrum, age) drive everything downstream but have distinct physics (nuclear evolution, HR diagram).  
**Responsibilities:** Star formation parameters, main-sequence evolution (simplified analytic → later detailed), stellar spectrum for climate boundary conditions.

#### `worldsmith-planet`
**Why separated:** Planetary bulk properties—mass, radius, orbit, rotation, magnetic field seed—form the bridge between stellar and surface domains.  
**Responsibilities:** Orbital mechanics, tidal evolution, bulk composition, differentiation (core/mantle/crust mass fractions), rotation period.

#### `worldsmith-geology`
**Why separated:** Solid-planet processes operate on million-year timescales with distinct equations (plate tectonics, volcanism, erosion).  
**Responsibilities:** Crust/mantle thermal state, topography generation via tectonics, volcanic outgassing fluxes → atmosphere.

#### `worldsmith-atmosphere`
**Why separated:** Fluid envelope physics (composition, pressure structure, escape) is independent of crust deformation.  
**Responsibilities:** Atmospheric composition, vertical structure, escape rates, greenhouse opacity → climate boundary.

#### `worldsmith-climate`
**Why separated:** Energy balance and circulation require atmosphere + surface albedo + stellar input; distinct time step from geology.  
**Responsibilities:** Radiative balance, latitudinal energy transport, surface temperature/precipitation fields, ice albedo feedback.

#### `worldsmith-chemistry`
**Why separated:** Reaction networks (outgassing, photochemistry, ocean chemistry) are reusable across atmosphere, ocean, and future biosphere.  
**Responsibilities:** Element abundance tracking, equilibrium/kinetic solvers, reaction networks as declarative data.

---

### Orchestration Layer

#### `worldsmith-state`
**Why it exists:** One authoritative container for all model data at the current simulation time.  
**Why separated:** Modules must not hold hidden global state. The state object is passed explicitly.  
**Responsibilities:** `WorldState` struct, field registry, time coordinates (simulation clock, epoch markers), dirty flags.

#### `worldsmith-engine`
**Why it exists:** Coordinates pipeline execution, scheduling, and module registration.  
**Why separated:** Domain modules know physics; the engine knows *order* and *timing*.  
**Responsibilities:** Pipeline runner, multi-rate scheduler, event dispatch, command handler (from UI/CLI), determinism audit log.

---

### Presentation Layer

#### `worldsmith-visualization`
**Why separated:** Translating simulation fields into GPU-ready meshes, colormaps, and isosurfaces is not simulation and not raw rendering.  
**Responsibilities:** Colormap selection, isobar/isotherm extraction, streamlines, LOD decimation, `VisualSnapshot` construction.

#### `worldsmith-render`
**Why separated:** GPU pipeline, shaders, and frame loop must never pull in simulation solvers.  
**Responsibilities:** wgpu device/surface, shader management, globe rendering, field overlay passes, camera.

#### `worldsmith-ui`
**Why separated:** Panels, menus, and inspectors are application chrome—not simulation.  
**Responsibilities:** Parameter editing (sends commands), timeline scrubber, layer toggles, module debug views.

---

### I/O Layer

#### `worldsmith-io`
**Why separated:** Save/load lifecycle (atomic writes, autosave, corruption recovery) is orthogonal to physics.  
**Responsibilities:** Save file read/write, checkpoint management, load validation.

#### `worldsmith-export`
**Why separated:** Export formats serve external tools; they should not bloat the simulation or render crates.  
**Responsibilities:** VTK, GeoTIFF, CSV time series, JSON schema export for reproducibility bundles.

---

### Application Layer

#### `worldsmith-cli`
Headless pipeline execution, batch seeds, benchmark mode, export-only runs.

#### `worldsmith-app`
Composes engine + render + ui into the desktop planetary laboratory.

---

## Simulation Architecture

### Execution Model

WorldSmith uses a ** staged pipeline** for generation and a ** multi-rate tick loop** for ongoing simulation.

```
Generation (forward-only, coarse → fine):
  Stellar → Planet → Geology (bulk) → Atmosphere → Climate → Geology (surface detail)

Runtime (multi-rate):
  ┌─ Orbital tick     (hours–years)     ─ planet
  ├─ Climate tick     (hours–days)      ─ climate, atmosphere (fast chemistry)
  ├─ Geological tick  (years–Myr)       ─ geology
  ├─ Stellar tick     (Myr–Gyr)         ─ stellar (slow)
  └─ Chemistry tick   (coupled to parent module's rate)
```

### Pipeline Stage Contract

Every stage implements `PipelineStage` (defined in `worldsmith-traits`):

```rust
// Conceptual — not implemented yet
trait PipelineStage {
    fn id(&self) -> StageId;
    fn dependencies(&self) -> &[StageId];
    fn execute(&self, ctx: &mut StageContext) -> StageResult;
}
```

- **Input:** Read-only access to dependency stage outputs via `StageContext`.
- **Output:** Writes to named fields in `WorldState`.
- **Failure:** Returns typed errors (non-convergence, invalid boundary)—never silent fallback values.

### Multi-Rate Scheduler

The engine maintains a **priority queue of scheduled events** keyed by simulation time. Each module registers its next wake time. The scheduler advances global time to the earliest event, runs the corresponding module tick, and repeats.

This avoids forcing geology to run every climate timestep (wasteful) or climate to miss rapid orbital forcing (wrong).

### Determinism Contract

1. Master seed in `WorldState`.
2. Each module receives `RngStream::derive(module_id, tick_index)`.
3. Parallel grid operations use deterministic tile ordering (row-major index split).
4. Floating-point: IEEE 754 with fixed reduction order; document any platform-sensitive paths.

---

## Planet Generation Pipeline

Phase 1 generation is **forward causal chain**—each stage consumes upstream physics outputs.

```
┌──────────────┐
│ 1. PRESET    │  User selects or defines initial conditions
│    LOAD      │  (stellar mass, metallicity, disk mass, seed)
└──────┬───────┘
       ▼
┌──────────────┐
│ 2. STELLAR   │  Luminosity, effective temperature, spectrum,
│    FORMATION │  age, stellar wind parameters
└──────┬───────┘
       ▼
┌──────────────┐
│ 3. PLANetary │  Orbital distance, eccentricity, planetary mass,
│    ACCRETION │  bulk composition, differentiation, rotation
└──────┬───────┘
       ▼
┌──────────────┐
│ 4. GEOLOGY   │  Internal heat flow, crust thickness, initial
│    (BULK)    │  topography seed, tectonic regime
└──────┬───────┘
       ▼
┌──────────────┐
│ 5. ATMOSPHERE│  Outgassing + escape equilibrium, composition,
│              │  surface pressure, greenhouse gases
└──────┬───────┘
       ▼
┌──────────────┐
│ 6. CLIMATE   │  Surface temperature field, circulation cells,
│              │  precipitation, ice lines, albedo
└──────┬───────┘
       ▼
┌──────────────┐
│ 7. GEOLOGY   │  Erosion/deposition driven by climate;
│    (SURFACE) │  river networks, sediment, volcanic resurfacing
└──────┬───────┘
       ▼
┌──────────────┐
│ 8. SNAPSHOT  │  Build VisualSnapshot + optional export bundle
│    EMIT      │
└──────────────┘
```

**Note:** Stage 7 loops back to modify geology based on climate—a controlled feedback within the generation pipeline, not ad-hoc noise.

---

## Data Flow & State Management

### Authoritative State

`WorldState` (in `worldsmith-state`) is the **single mutable source of truth** during simulation:

```
WorldState
├── metadata      (seed, version, sim_time, epoch)
├── stellar       (StellarSystem)
├── planets[]     (PlanetBody per planet)
│   ├── bulk      (mass, radius, orbit, rotation)
│   ├── geology   (GeologyState + grid fields)
│   ├── atmosphere(AtmosphereState + vertical profiles)
│   └── climate   (ClimateState + surface fields)
├── chemistry     (global element pools, per-body networks)
└── registry      (field name → typed buffer handle)
```

### Read Path (Simulation → Screen)

```
WorldState
    │  (on demand, or after tick)
    ▼
VisualSnapshot  ← immutable, cheap to clone/share with render thread
    │  meshes, textures, scalar ranges, metadata for UI
    ▼
worldsmith-render  →  framebuffer
worldsmith-ui      →  inspector panels
```

### Write Path (UI → Simulation)

```
UI widget change
    ▼
EngineCommand  (enum: SetParameter, Step, Pause, LoadPreset, …)
    ▼
worldsmith-engine  validates → mutates WorldState OR schedules pipeline re-run
```

**UI never holds a mutable reference to `WorldState`.**

### Cross-Module Data Sharing

Modules do **not** call each other directly. They communicate through:

1. **Shared state fields** — producer writes field; consumer reads in its next tick.
2. **Typed events** — e.g., `GeologyEvent::Outgassing { flux, composition }` dispatched by engine after geology tick; atmosphere module applies on next wake.
3. **Pipeline stage outputs** — during generation, explicit stage ordering guarantees availability.

---

## Module Communication

### 1. Field Registry

Named typed fields (temperature, elevation, pressure) live in a registry. Modules declare `reads()` and `writes()` field sets for scheduling validation.

### 2. Event Bus

Synchronous, typed event queue processed between ticks:

| Event | Publisher | Subscriber |
|-------|-----------|------------|
| `OutgassingFluxChanged` | geology | atmosphere, chemistry |
| `InsolationChanged` | stellar, planet | climate |
| `SurfaceAlbedoChanged` | climate, geology | climate |
| `ImpactEvent` | (future) planet | geology, atmosphere |

Events are **records**, not callbacks—preserves determinism and testability.

### 3. Command Channel

External inputs (UI, CLI, scripts) enqueue `EngineCommand`s processed at tick boundaries.

### 4. Snapshot Channel

After selected ticks, engine builds `VisualSnapshot` and sends via channel to render thread (double-buffered).

---

## Rendering & Visualization Separation

| Layer | Knows about | Must not know about |
|-------|-------------|---------------------|
| **visualization** | Models, colormaps, mesh algorithms | wgpu, shaders, windows |
| **render** | VisualSnapshot, GPU | Solvers, WorldState mutation |
| **ui** | Commands, read-only snapshots | GPU details, field solvers |

This allows:
- Headless CI without GPU.
- Swap wgpu → other backend later.
- Run simulation on server, stream snapshots to viewer (future).

---

## Plugin & Extension Model

Future modules (biosphere, civilization, binary stars, moons, ocean circulation) plug in without engine rewrites.

### Adding a Domain Module

1. Create crate `worldsmith-<domain>`.
2. Implement `SimulationModule` + relevant `PipelineStage`s.
3. Define new model types in `worldsmith-models` (or sub-module).
4. Register in engine via `EngineBuilder::register_module()`.
5. Declare field reads/writes and event subscriptions.
6. Add visualization layer in `worldsmith-visualization`.
7. Add UI panel in `worldsmith-ui` (optional).

### Extension Points

| Extension Point | Location | Purpose |
|-----------------|----------|---------|
| `PipelineStage` | `worldsmith-traits` | Generation-time steps |
| `SimulationModule` | `worldsmith-traits` | Runtime ticks |
| `VisualLayer` | `worldsmith-traits` | New renderable field overlays |
| `ExportFormat` | `worldsmith-traits` | New export targets |
| `PresetProvider` | `worldsmith-presets` | New initial condition bundles |
| Reaction networks | data files in `assets/chemistry/` | Chemistry without recompile |

### Dependency Rule for Plugins

```
worldsmith-<new-domain>
  → worldsmith-models, worldsmith-traits, worldsmith-math, …
  → NOT → worldsmith-render, worldsmith-ui, worldsmith-app
```

---

## Naming Conventions

### Crates
- Prefix: `worldsmith-`
- Domain names: singular noun (`geology`, not `geologies`)
- Kebab-case: `worldsmith-atmosphere`

### Rust Types
- Structs: `PascalCase` — `PlanetBody`, `AtmosphereState`
- Enums: `PascalCase` — `EngineCommand`, `GeologyEvent`
- Traits: `PascalCase` — `SimulationModule`, `PipelineStage`
- Functions: `snake_case` — `advance_orbit`, `compute_insolation`
- Constants: `SCREAMING_SNAKE_CASE` — `STEFAN_BOLTZMANN`

### Fields & Grids
- Grid fields: `snake_case` — `surface_temperature`, `elevation`
- IDs: `{Domain}{Entity}` — `PlanetId`, `StarId`, `StageId`

### Files
- One primary type or concern per file.
- Module roots: `lib.rs` re-exports public API only.
- Tests: `#[cfg(test)] mod tests` in-file for unit; `tests/` for integration.

### Shaders
- `assets/shaders/<pass_name>.wgsl`

### Presets
- `assets/presets/<category>/<name>.ron`

---

## File Responsibilities

### Per-Crate Layout

```
worldsmith-<name>/
├── Cargo.toml          # Dependencies (minimal until implementation)
├── README.md           # Crate purpose, public API overview, dependencies
└── src/
    ├── lib.rs          # Public re-exports
    ├── <module>.rs     # One concern per file
    └── ...
```

### Repository Root

| File | Responsibility |
|------|----------------|
| `ARCHITECTURE.md` | This document |
| `README.md` | Project overview, build instructions (when implemented) |
| `Cargo.toml` | Workspace manifest |
| `docs/adr/` | Architecture Decision Records |
| `docs/design/` | Deep dives (data flow, pipeline, plugins) |
| `assets/` | Shaders, presets, chemistry networks (data, not code) |
| `tools/` | Code generators, schema validators (future) |

---

## Future Scalability

### Near-Term (Phase 2–4)
- GPU compute shaders for climate grid ops (via wgpu compute, driven by `worldsmith-climate`).
- Adaptive mesh refinement for geology hot spots.
- Parallel pipeline stages where dependencies allow (stellar + disk structure).

### Medium-Term
- **Binary/multi-star:** Extend `StellarSystem` model; add `worldsmith-stellar` submodule—no changes to render.
- **Moons:** `PlanetBody` hierarchy (parent body, children); orbital module handles N-body.
- **Ocean model:** New crate reading climate + geology boundary conditions.
- **Biosphere:** Subscribes to chemistry + climate events; adds fields, not engine forks.

### Long-Term
- Distributed simulation (domain decomposition)—`WorldState` sharding by body/region.
- Web viewer (WASM render crate consuming snapshots).
- Python bindings (`worldsmith-py`) for research scripting—thin FFI over `worldsmith-engine`.
- Replay system: event log + initial state → bit-identical reproduction.

### Performance Strategy
- Hot loops in Rust with SIMD where proven necessary.
- Field buffers: Structure-of-Arrays for cache-friendly grid ops.
- Render thread always decoupled—simulation never waits on vsync.

---

## Folder Structure

See the generated tree in the repository root. Summary:

```
WorldSmith/
├── ARCHITECTURE.md
├── README.md
├── Cargo.toml                 # workspace
├── assets/
│   ├── chemistry/             # reaction network data files
│   ├── presets/               # .ron preset files
│   └── shaders/               # .wgsl shaders
├── crates/                    # 22 focused library/application crates
├── docs/
│   ├── adr/
│   └── design/
├── tests/
│   └── integration/
└── tools/
    └── schema/
```

Each crate under `crates/` follows the per-crate layout above with `src/` ready for implementation.

---

## Next Steps (Phase 2)

Implementation order—each step is a vertical slice with tests:

1. `worldsmith-math`, `worldsmith-units`, `worldsmith-rng`
2. `worldsmith-models`, `worldsmith-state`, `worldsmith-traits`
3. `worldsmith-engine` (empty pipeline, one no-op stage)
4. `worldsmith-stellar` (analytic main-sequence stub with real equations)
5. Determinism golden test harness
6. Presentation shell (`visualization` → `render` → `app`) displaying a colored sphere from snapshot

No stage advances until the previous compiles, tests, and documents its public API.
