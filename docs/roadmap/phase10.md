# Phase 10 Roadmap — Planet Evolution Framework

## Summary
Phase 10 introduces a **continuous, multi-rate evolution framework** so that
planets change over millions to billions of years.  Each scientific discipline
becomes an independent `SimulationModule` inside a new `worldsmith-evolution`
crate.  The engine's existing scheduler and snapshot mechanism are reused
unchanged.

---

## Proposed Architecture
- **Crate:** `worldsmith-evolution` — hosts discipline modules.
- **Scheduler:** multi-rate absolute-timestamp scheduler; modules register
  tick intervals.
- **Modules:** Core, Mantle, Tectonics, Volcanism, Atmosphere, Hydrology,
  Climate, MagneticField, Biosphere.
- **State:** Long-lived planetary fields live in `Planet`; high-resolution
  grids stay module-local.
- **Communication:** typed `SimulationEvent`s and shared `WorldState` fields.
- **Determinism:** all modules are pure functions of `timestamp_s` + `seed`.

---

## Module Dependency Graph
```
worldsmith.evolution.core
    └── worldsmith.evolution.mantle
            ├── worldsmith.evolution.tectonics
            └── worldsmith.evolution.volcanism
                    └── worldsmith.evolution.atmosphere
                            ├── worldsmith.evolution.hydrology
                            └── worldsmith.evolution.climate
                                    └── worldsmith.evolution.biosphere
```

---

## Evolution Timeline
1. **Phase 10A — Core and Mantle**
   - Introduce `worldsmith-evolution` crate and `SimulationModule` extension tests.
   - `CoreEvolutionModule`, `MantleModule`.
   - New `Planet` model fields: `core_temperature`, `mantle_temperature`.
   - Benchmark and determinism harness.

2. **Phase 10B — Surface & Atmosphere**
   - `TectonicsModule`, `VolcanismModule`, `AtmosphereModule`.
   - New `Planet` fields: `tectonic_activity`, `volcanic_flux`,
     `atmospheric_pressure`, `atmospheric_composition`.
   - Event chain: volcanism → atmosphere pressure.

3. **Phase 10C — Hydrosphere & Climate**
   - `HydrologyModule`, `ClimateModule`.
   - New `Planet` fields: `ocean_mass`, `ice_cover_fraction`,
     `surface_temperature`.
   - Climate grid integration with `worldsmith-grid` for latitudinal bins.

4. **Phase 10D — Magnetic Field & Biosphere**
   - `MagneticFieldModule`, `BiosphereModule`.
   - New `Planet` fields: `magnetic_field_strength`, `habitability_index`.
   - Event chain: biosphere → atmospheric O2 fraction via `Custom` events.

5. **Phase 10E — Extensibility & Polish**
   - Hook points for user scripts, terraforming, impacts.
   - Save/load support for new fields (schema version bump).
   - Full golden tests + performance regression suite.

---

## State Ownership
- **Planet** owns all durable properties (see ADR 010 for the full table).
- **Module-local buffers** own transient per-tick scratch space.
- **WorldState** remains the single mutable container; modules do not nest
  hidden global state.

---

## Expected Implementation Phases

| Phase | Deliverable | Difficulty |
|-------|-------------|------------|
| 10A | Core + mantle modules, new crate, determinism harness | Medium |
| 10B | Tectonics + volcanism + atmosphere, event chains | Medium-High |
| 10C | Hydrology + climate, grid integration | High |
| 10D | Magnetic field + biosphere | Medium |
| 10E | Extensibility hooks, save/load, benchmarks | Medium |
| **Overall** | | **High** |

---

## Estimated Implementation Difficulty
**High.**  High because multi-rate scheduling across 100k planets must remain
deterministic and fast, and because science validation requires careful
parameterization per discipline.  Most individual modules are **medium**
complexity in isolation; the challenge is integration boundaries and performance
at scale.

## Major Risks
1. **Determinism across modules** — solved by absolute timestamps and no
   interior mutability.
2. **Performance at 10k–100k planets** — solved by `rayon` and grid LOD.
3. **Scientific accuracy without code rip** — solved by typed events, explicit
   contracts, and versioned field schemas.

## Recommended Phase 10A
Build `worldsmith-evolution` crate, extract the `evolve_planet` monolithic
function into `CoreEvolutionModule` + `MantleModule`, add the new `Planet`
fields, and implement the parallel determinism harness.

## Confidence Level
**High.**  The existing engine pipeline, event system, and multi-rate scheduler
already provide all the architectural primitives needed.  Phase 10 is primarily
a decomposition and state-modeling exercise.
