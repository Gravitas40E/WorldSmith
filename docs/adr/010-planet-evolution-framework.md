# ADR 010: Planet Evolution Framework

## Status
Proposed — Phase 10 architecture design.

## Context
Planet evolution currently happens once during `PlanetEvolutionModule::initialize()`.
The monolithic `worldsmith-planet::evolution::evolve_planet` function runs a fixed pipeline
- interior differentiation
- atmosphere generation
- climate feedback
- weather / hydrology
- geology
- habitability

and then never runs again.

For a planetary laboratory we need continuous evolution: planets must cool, atmospheres escape, climates shift, and biospheres change over millions of years.  Each scientific discipline must be an independently pluggable component so that adding life, civilization, or terraforming never touches existing geology or atmosphere code.

## Decision

### 1. New crate: `worldsmith-evolution`
Create a dedicated `worldsmith-evolution` crate.  It becomes the **domain simulation module host** for all long-term planetary science.  Why separate:
- Keeps `worldsmith-planet` focused on formation, bulk composition, and orbital/rotational models.
- Allows future `worldsmith-geology`, `worldsmith-atmosphere`, `worldsmith-climate`,
  `worldsmith-hydrosphere`, and `worldsmith-biosphere` crates to share traits and
  utilities without circular dependencies.
- Matches the Domain Simulation Layer defined in `ARCHITECTURE.md`.

Each evolution discipline is implemented as a `SimulationModule` + `PipelineStage`
pair.  New modules register with the engine at run time—no existing module
source code changes are required.

### 2. Time model: multi-rate absolute-timestamp scheduler
Engines already accumulate `timestamp_s` and offer `delta_seconds`.  Evolution
modules declare their required tick interval in the module descriptor
(e.g., 1 day for climate, 1 year for surface geology, 1 million years for
mantle cooling).  The scheduler advances time by the minimum wake-time delta
and runs only due modules.

| Concern | Recommended tick interval | Rationale |
|---------|--------------------------|-----------|
| Core / mantle | 0.1 – 2 Myr | Thermal diffusion slow |
| Tectonics / volcanism | 0.1 – 1 Myr | Plate-cycle timescale |
| Surface geology / erosion | 1 kyr – 100 kyr | Climate-driven denudation |
| Atmosphere | 1 year | Thermal + escape equilibrium |
| Ocean / hydrology | 1 month | Seasonal water cycle |
| Climate | 1 day | Orbital + diurnal forcing |
| Biosphere | 1 year | Ecological turnover |

**Deterministic replay:** because state is a pure function of
`timestamp_s` + `seed`, replay is identical to simply re-running the same
timestamp sequence.  No integration history is required.

**Fast-forward:** UI or batch tools set the target simulation time.  The
scheduler runs all due ticks in timestamp order until the target is reached.
Modules are free to sub-step internally if their equations require it
(e.g., Runge-Kutta for atmospheric diffusion), but sub-steps must be bounded
and deterministic.

### 3. Module ordering & dependency graph
Modules execute only after their dependencies have finished the same tick.
Dependencies are declared via `PipelineStageDescriptor::dependencies()`.

```
worldsmith.evolution.core          (priority: 100)
    └── worldsmith.evolution.mantle            (50)
            ├── worldsmith.evolution.tectonics  (40)
            └── worldsmith.evolution.volcanism  (35)
                    └── worldsmith.evolution.atmosphere  (30)
                            ├── worldsmith.evolution.hydrology   (28)
                            └── worldsmith.evolution.climate      (25)
                                    └── worldsmith.evolution.biosphere (20)
```

**Why bottom-up:**
1. **Core** produces internal heat budget and radiogenic inventories.
2. **Mantle** consumes core heat flux to compute convection and thermal state.
3. **Tectonics / Volcanism** consume mantle state.
4. **Atmosphere** consumes volcanic outgassing (and stellar/insolation input).
5. **Hydrology** depends on surface temperature and atmospheric pressure.
6. **Climate** couples atmosphere + insolation + albedo + hydrology.
7. **Biosphere** is last—life depends on climate, water, and atmosphere.

The graph is linearizable; there are no circular dependencies.  If a future
module (e.g., life-driven weathering) must later feed back into atmosphere or
tectonics, it does so by publishing `SimulationEvent`s consumed on the next
tick, not by direct cross-module calls.

### 4. Planet state ownership

Long-lived planetary properties that belong in `Planet` (authoritative, snapshotted):

| Property | Category | Rationale |
|----------|----------|-----------|
| `core_temperature` | thermal | Needed by magnetic field, tectonics, and visualization |
| `mantle_temperature` | thermal | Drives volcanism and tectonics |
| `tectonic_activity` | geology | Visible surface property; consumed by render/UI |
| `volcanic_flux` | mass/energy | Outgassing source for atmosphere |
| `atmospheric_pressure` | atmosphere | Fundamental climate boundary |
| `atmospheric_composition` | atmosphere | Greenhouse, chemistry, biosphere |
| `surface_temperature` | climate | Output of climate, input to hydrology |
| `ocean_mass` | hydrosphere | Visualizable; affects climate heat capacity |
| `ice_cover_fraction` | surface | Albedo feedback, visible |
| `magnetic_field_strength` | core/dynamo | Observable from space |
| `habitability_index` | biosphere | Composite output for UI/science |

These extend the existing `Planet` model fields already introduced in Phase 9.
They are **stored permanently** so that:
- `SimulationSnapshot` carries them automatically.
- UI and render can read them without invoking module logic.
- Deterministic save/reload preserves them.

#### Derived vs stored tradeoffs
| Strategy | Advantage | Disadvantage |
|----------|-----------|--------------|
| **Store permanently in `Planet`** | Single source of truth; snapshots work; UI reads cheap | Module must keep them current; schema churn |
| **Compute on demand from raw buffers** | No stale data from partial updates | Every consumer duplicates solver code; determinism suffers |

**Recommendation:** store all coarse-grained planetary state in `Planet`.
High-resolution grid fields (temperature maps, wind vectors, composition grids)
live in module-local buffers and are exported only on demand or when writing
snapshots that explicitly include them.

### 5. Module boundaries & contracts
Every discipline module:
- Implements `SimulationModule` (initialize/update/shutdown).
- Declares `reads()` and `writes()` field keys for scheduling diagnostics.
- Publishes immutable `SimulationEvent`s when something significant changes
  (e.g., `VolcanicEruption`, `AtmosphericPressureChanged`).
- Consumes events emitted earlier in the same tick or previous tick.

No module calls another module directly.  All communication is through
`WorldState` and the event queue.

### 6. Performance scaling

| World count | Expected per-tick cost | Strategy |
|-------------|------------------------|----------|
| 100 | < 1 ms | Naive loop fine |
| 1,000 | 1–10 ms | `rayon::par_iter()` over `WorldState::planets` |
| 10,000 | 50–200 ms | Parallel + SIMD for vector math; grid modules already use `worldsmith-grid` halos |
| 100,000 | Seconds | Spatial partitioning; modules accept `&[PlanetId]` subsets; GPU offload path via storage buffers |

Because every module iterates `WorldState` independently, the scheduler can
split the planet list into chunks for different threads when the count crosses
a threshold.  No module interface changes to add parallelism.

### 7. Extensibility hooks

New systems plug in by creating a new `SimulationModule` and registering it:

- **Life / BiosphereModule** — reads climate, ocean, atmosphere; writes
  `habitability_index`, biomass, O2 fraction.  Depends on `climate`.
- **CivilizationsModule** — reads habitability and surface pressure; writes
  technology level, emissions.  Depends on `biosphere`.
- **TerraformingModule** — reads atmosphere, temperature; writes engineered
  parameters.  Depends on `climate` and `atmosphere`.
- **SatelliteModule** — extends the orbital framework to artificial satellites.
  Reuses `OrbitalDynamicsModule` logic; adds `Satellite` model type.
- **Planetary EngineeringModule** — user-driven parameter editor; emits
  `EngineCommand`s rather than mutating state directly.

None of these modify existing geology, atmosphere, or climate modules.

## Alternatives Considered
- **Keep monolithic `PlanetEvolutionModule`** — rejected: impossible to evolve
  independently; change to one discipline breaks all.
- **Use generation-time `PipelineStage`s** — rejected: can't run continuously
  over geological time.
- **Signal-slot callbacks between modules** — rejected: breaks determinism
  and serialization; typed events are preferred.

## Risks
- **Stale state from out-of-order reads** — mitigations: module-local state
  is always derived from `WorldState` at `timestamp_s`; no cached parent
  positions survive a tick.
- **Performance at 100k planets** — mitigations: parallel planet iterators
  inside each module; future grid LOD and GPU dispatch paths.
- **Scientific accuracy** — mitigations: each module publishes its
  assumptions in rustdoc; configuration validation catches impossible inputs.

## Summary
Phase 10 replaces single-shot planet evolution with a multi-rate, modular
framework inside `worldsmith-evolution`.  Each discipline is an independent
`SimulationModule` with explicit dependencies, deterministic time semantics,
and typed event communication.  Planet state becomes the durable snapshot of
all planetary properties; high-resolution grids remain module-local.
