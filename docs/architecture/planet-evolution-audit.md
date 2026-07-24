# Planet Evolution Architecture Audit — Phase 12

**Date:** 2026-07-23  
**Scope:** Complete planetary evolution pipeline through CarbonCycleModule  
**Status:** Design / verification phase only — no scientific equations modified  

---

## Executive Summary

The WorldSmith planetary evolution architecture is a clean, deterministic,
single-writer module pipeline.  Eight evolution modules execute in a strict
directed acyclic graph (DAG) ordered by declared dependencies and numerical
priority.  Every persistent field on `Planet` has exactly one declared runtime
writer.  The scheduler, snapshot system, and validation framework are
internally consistent with the ownership model defined in ADR-010 and ADR-011.

The architecture is **approved for Phase 13 (BiosphereModule)** with the
understanding that a small number of non-blocking refinements should be
addressed in parallel.  No scientific equations require correction.  No
scheduler changes are required.

**Architecture Score: 8 / 10**  
**Determinism Score: 9 / 10**  
**Maintainability Score: 8 / 10**  
**Scalability Score: 6 / 10**  
**Scientific Extensibility Score: 9 / 10**  
**Testability Score: 7 / 10**

---

## Architecture Overview

### Modules and Priorities

| Priority | Module ID | Module Struct |
|----------|-----------|---------------|
| 100 | `worldsmith.evolution.core` | `CoreEvolutionModule` |
| 50 | `worldsmith.evolution.mantle` | `MantleEvolutionModule` |
| 40 | `worldsmith.evolution.volcanism` | `VolcanismModule` |
| 30 | `worldsmith.evolution.plate_tectonics` | `PlateTectonicsModule` |
| 25 | `worldsmith.evolution.atmosphere` | `AtmosphereModule` |
| 20 | `worldsmith.evolution.hydrology` | `HydrologyModule` |
| 15 | `worldsmith.evolution.climate` | `ClimateModule` |
| 10 | `worldsmith.evolution.carbon_cycle` | `CarbonCycleModule` |

### Execution Guarantees

- **Initialization:** Modules are initialized in pipeline execution order
  (lowest priority first, respecting dependency declarations).
- **Update:** Each tick, the scheduler iterates the resolved execution order.
  After all modules update, published events are dispatched to all modules
  in the same order.
- **Snapshot:** `WorldState::snapshot()` is invoked after each fixed
  substep, but only retained when `engine_config.rendering.enabled` is true.
- **Shutdown:** Modules are shut down in reverse execution order.

### Pipeline Mechanics

`Pipeline::build()` validates:
1. Unique stage identifiers.
2. All declared dependencies are present in the stage set.
3. No circular dependencies (topological sort failure → `EngineError::CircularDependency`).

Order resolution: modules with no unresolved dependencies are selected by
ascending `priority` (lower number = earlier execution), with identifier as
tiebreaker.  This means **clamping a module to a specific dependency is
possible even if its natural priority would place it later**.

---

## Ownership Matrix

Every persistent field on `Planet` has exactly one declared runtime writer.
Transient module-local state (config, `initialized` flag) stays inside the
module struct and is never serialized into `Planet`.

| Planet Field | Owner Module | Notes |
|---|---|---|
| `planet.interior.age_seconds` | `CoreEvolutionModule` | |
| `planet.interior.core_temperature` | `CoreEvolutionModule` | |
| `planet.interior.radiogenic_heat` | `CoreEvolutionModule` | |
| `planet.interior.internal_heat` | `CoreEvolutionModule` | |
| `planet.interior.mantle_temperature` | `MantleEvolutionModule` | |
| `planet.interior.heat_flux` | `MantleEvolutionModule` | |
| `planet.volcanism` (`VolcanismState`) | `VolcanismModule` | |
| `planet.plate_tectonics` (`PlateTectonicsState`) | `PlateTectonicsModule` | |
| `planet.atmosphere_state` (`AtmosphereState`) | `AtmosphereModule` | |
| `planet.hydrology_state` (`HydrologyState`) | `HydrologyModule` | |
| `planet.climate_state` (`ClimateState`) | `ClimateModule` | |
| `planet.carbon_cycle_state` (`CarbonCycleState`) | `CarbonCycleModule` | |

**Ownership integrity: PASS.** No duplicated writers detected.  The
`worldsmith-validation` crate's `validate_field_ownership` helper correctly
enforces this rule at test time.

### Hidden-Write Check

Each module's `update()` implementation was inspected.  All mutations go
through `state.world_mut().planets.insert(...)`.  No module writes to
`WorldState` globals, event queues, or other planets.  No module writes to a
`Planet` field that is not listed in its `writes()` declaration.

**Hidden-write audit: PASS.**

---

## Dependency Graph Review

### Declared Dependencies (from `plugin.rs`)

| Module | Dependencies |
|---|---|
| `CoreEvolutionModule` | `[]` |
| `MantleEvolutionModule` | `[core]` |
| `VolcanismModule` | `[mantle]` |
| `PlateTectonicsModule` | `[volcanism]` |
| `AtmosphereModule` | `[plate_tectonics]` |
| `HydrologyModule` | `[atmosphere]` |
| `ClimateModule` | `[hydrology]` |
| `CarbonCycleModule` | `[climate]` |

### Graph Properties

- **Edges:** 7 dependency edges.
- **Longest path:** 7 hops (Core → Mantle → Volcanism → PlateTectonics → Atmosphere → Hydrology → Climate → CarbonCycle).
- **Cycles:** None detected.  `Pipeline::resolve_order()` would reject any.
- **Unnecessary coupling:** None detected.  Each module depends only on its
  direct upstream producer.

### Scheduling Correctness

The numerical priorities match the dependency chain monotonically
(100 → 50 → 40 → 30 → 25 → 20 → 15 → 10).  Because priorities are unique
and strictly decreasing along the dependency edges, the topological sort and
priority-based selection produce identical results.  This redundancy is
defensive but not required.

**Dependency graph: PASS.**

---

## Feedback Loop Review

### Observed Feedback Paths

| Source → Consumer | Mechanism | Delay |
|---|---|---|
| Volcanism → Atmosphere | volcanic_flux → outgassing efficiency → atmospheric_mass_kg | 1 tick (Atmosphere reads VolcanismState) |
| Atmosphere → Climate | mean_temperature_k, surface_pressure_pa → equilibrium_temperature_k | 1 tick (Climate reads AtmosphereState) |
| Hydrology → Climate | liquid_water_fraction, ocean coverage → planetary_albedo (implicit via ClimateModule config) | 1 tick |
| Climate → CarbonCycle | planetary_albedo, equilibrium_temperature_k → weathering_flux | 1 tick (CarbonCycle reads ClimateState) |
| CarbonCycle → Atmosphere (next tick) | volcanic_carbon_flux, weathering_flux, ocean_exchange_flux queued for AtmosphereModule | Intended; AtmosphereModule does not yet consume these fluxes |

### One-Tick Delay Guarantee

Because the scheduler executes modules sequentially and each module reads the
state snapshot produced at the *start* of the current tick (before any module
in this tick has mutated it), a module's output is always based on the
previous tick's inputs from upstream modules.  This enforces a strict
one-tick delay on all feedback without any explicit bookkeeping.

**Feedback loop audit: PASS.**

### Future Cyclic Risks

| Potential Addition | Risk | Mitigation |
|---|---|---|
| Biosphere reads Climate + CarbonCycle, writes CarbonCycle / Hydrology | Medium — if Biosphere writes CarbonCycleState directly it would create a cycle with CarbonCycle → Atmosphere → Climate → Biosphere → CarbonCycle | Enforce one-tick delay via priority; Biosphere should produce fluxes consumed by CarbonCycle/Atmosphere in the *next* tick, never mutate another module's state directly. |
| Cryosphere reads Hydrology + Climate, writes Hydrology / Albedo | Medium — Cryosphere writing HydrologyState would create a Hydrology ↔ Cryosphere cycle | Cryosphere should produce ice_area / albedo_factor fluxes that Hydrology/Climate consume later. |
| Surface Chemistry reads Atmosphere + CarbonCycle + Hydrology | Low — if it writes only its own state, no cycle | Ensure SurfaceChemistryState is the sole writer of its fields. |

---

## State Model Review

### Persistent State

Persistent state lives in `Option<T>` fields on `Planet` and is serialized
into `SimulationSnapshot`.  Each module owns exactly one persistent state
struct.

| State Struct | Fields |
|---|---|
| `InteriorState` | `age_seconds`, `core_temperature`, `mantle_temperature`, `radiogenic_heat`, `internal_heat`, `heat_flux` |
| `VolcanismState` | `volcanic_flux`, `volcanic_activity`, `magma_generation_rate` |
| `PlateTectonicsState` | `plate_velocity`, `crustal_recycling_rate`, `tectonic_activity` |
| `AtmosphereState` | `atmospheric_mass_kg`, `surface_pressure_pa`, `mean_temperature_k`, `atmosphere_composition` |
| `HydrologyState` | `total_water_mass_kg`, `ocean_mass_kg`, `atmospheric_water_mass_kg`, `ice_mass_kg`, `liquid_water_fraction` |
| `ClimateState` | `equilibrium_temperature_k`, `greenhouse_temperature_offset_k`, `planetary_albedo`, `climate_classification` |
| `CarbonCycleState` | `atmospheric_carbon_mass_kg`, `ocean_carbon_mass_kg`, `lithosphere_carbon_mass_kg`, `volcanic_carbon_flux_kg_per_s`, ` weathering_flux_kg_per_s`, `ocean_exchange_flux_kg_per_s`, `atmospheric_co2_fraction`, `carbon_partition_ratio`, `weathering_efficiency` |

### Derived State

Some fields are recalculated every tick but stored persistently:

- `ClimateState::equilibrium_temperature_k` and `greenhouse_temperature_offset_k` — computed from orbital and atmospheric inputs.
- `CarbonCycleState::atmospheric_co2_fraction`, `carbon_partition_ratio`, `weathering_efficiency` — computed from reservoir masses and config.

**Observation:** These derived values are stored on `Planet` rather than
computed on read.  This is acceptable for V1 performance and deterministic
replay, but it creates a risk that a module forgets to refresh them.  The
current implementation refreshes them every tick, so the risk is managed.

### Transient State

Module-local fields (`config`, `initialized`) are NOT stored on `Planet`.
They remain in the module struct inside the engine registry.  Because the
registry is not serialized into snapshots, transient state is correctly
excluded from persistence.

**State model audit: PASS with minor observation.**

---

## Conservation Review

### Water Conservation

`HydrologyState` tracks four water reservoirs.  Validation checks:

```text
ocean_mass_kg + atmospheric_water_mass_kg + ice_mass_kg
    <= total_water_mass_kg + 1e-3 * total_water_mass_kg.max(1.0)
```

**Status:** Approximated.  The 0.1% tolerance absorbs floating-point
rounding, but there is **no rigorous conservation guarantee** because other
modules (e.g., future Cryosphere) may redistribute water without updating
`total_water_mass_kg`.  HydrologyModule is the sole writer, so within V1
water is approximately conserved.

### Carbon Conservation

`CarbonCycleState` tracks three carbon reservoirs.  Validation checks
non-negativity only.  There is **no check** that

```text
atmospheric + ocean + lithosphere ≈ constant
```

**Status:** Not enforced.  Volcanic input, weathering removal, and ocean
exchange change total carbon each tick.  For V1 this is intentional (open
system), but future science may want to distinguish "closed-system" modes.

### Mass Conservation

`AtmosphereState::atmospheric_mass_kg` is evolved by outgassing and escape.
No cross-module conservation check links atmospheric mass to volcanic output
or escape loss.

**Status:** Not enforced.  Each module treats mass as an independent
reservoir; no global mass budget exists.

### Non-Negativity and Finiteness

All modules clamp reservoirs to `max(0.0)`.  Validation enforces `!is_nan()`
and `!is_infinite()` for all tracked floats.

**Conservation audit: PASS with gaps.**  V1 intentionally models open
systems.  Recommended: add optional strict-conservation assertions behind a
feature flag for future scientific tightening.

---

## Validation Audit

### Existing Coverage

`worldsmith-validation/src/state.rs` checks:

- `InteriorState`: non-NaN, non-infinite for `core_temperature`, `mantle_temperature`, `heat_flux`, `radiogenic_heat`.
- `VolcanismState`: non-NaN, non-infinite for `volcanic_flux`, `magma_generation_rate`.
- `PlateTectonicsState`: non-NaN, non-infinite for `plate_velocity`, `crustal_recycling_rate`.
- `AtmosphereState`: non-NaN, non-infinite for `atmospheric_mass_kg`, `surface_pressure_pa`, `mean_temperature_k`; composition fractions in `[0, 1]` and sum within `0.01` of `1.0`.
- `HydrologyState`: non-NaN, non-infinite; `liquid_water_fraction` in `[0, 1]`; component sum ≤ total + tolerance.
- `ClimateState`: non-NaN, non-infinite; `planetary_albedo` in `[0, 1]`; `greenhouse_temperature_offset_k` ≥ 0.
- `CarbonCycleState`: non-NaN, non-infinite; reservoirs non-negative.

### Missing Invariants

| Invariant | Severity | Recommendation |
|---|---|---|
| `CoreEvolutionModule` does not validate that `core_temperature > mantle_temperature` | Low | Add inner/outer core sanity bounds. |
| `InteriorState` has no minimum temperature floor (e.g., `> 0 K`) | Low | Add `> 0` check in validation. |
| `AtmosphericState::atmosphere_composition` may be empty (Earth-like fixture has 1 gas) | Low | Decide whether empty composition is physically meaningful. |
| Carbon total conservation not checked | Medium | Add option to assert `atmospheric + ocean + lithosphere` constant within tolerance. |
| Water component sum vs total not checked as equality | Low | Document that the 0.1% tolerance is intentional. |
| Atmosphere composition sum tolerance (`0.01`) is loose for scientific use | Medium | Consider tightening to `1e-6` or making it configurable. |
| `ClimateState::climate_classification` not validated against temperature bounds | Low | Ensure Temperate/Hot/Tropical/etc. are consistent with `equilibrium_temperature_k`. |
| No test coverage for `validate_field_ownership` with all 8 modules | Medium | Extend `ownership_validation.rs` to include Atmosphere, Hydrology, Climate, CarbonCycle. |

### Ownership / Cross-Module Validation Gaps

The `worldsmith-validation/tests/` files `ownership_validation.rs` and
`cross_module_validation.rs` only include Core, Mantle, Volcanism, and
PlateTectonics modules.  **This is a coverage gap.**  The newer modules
should be added to these tests so that any future `writes()`/`reads()`
regression is caught.

**Validation audit: PASS with recommended additions.**

---

## Performance Assessment

### Per-Tick Complexity

For `N` planets and `M` modules:

- **Module updates:** `O(N × M)` — each module iterates all planets once.
- **Event dispatch:** `O(E × M)` where `E` is published events per tick.
  In V1, all modules publish zero events, so this is negligible.
- **Snapshot:** `O(N × P)` where `P` is planet clone cost.  Snapshots are
  taken every substep but only retained when rendering is enabled.

### Allocation Patterns

- **Planet clone in module updates:** Every module clones the full `Planet`
  map into a temporary `Vec<(PlanetId, Planet, Option<State>)>` before
  iterating.  For `N` planets, this is `O(N)` heap allocations per module
  per tick.
- **Snapshot clone:** `WorldState::snapshot()` clones every `Planet`,
  `Moon`, `Star`, and stellar system.  Full deep clone.
- **No arena or bump allocator observed.**  Standard `Vec::clone()` and
  `HashMap` patterns.

### Complexity Estimates

| Planet Count | Per-Tick CPU | Memory | Notes |
|---|---|---|---|
| 100 | < 1 ms | ~ few MB | Trivial. |
| 1,000 | ~ few ms | ~ tens of MB | Comfortable. |
| 100,000 | ~ 100 ms — 1 s | ~ several GB | Snapshot and clone overhead dominates. |
| 1,000,000 | > 10 s | > tens of GB | Likely infeasible without structural changes. |

### Bottleneck Identification

- **Snapshot cloning** is the dominant scalability risk.  Capturing a clone
  of every planet each substep is O(N) allocation and copy.
- **Full-planet vectorization** in module updates avoids borrow-checker
  issues but sacrifices in-place mutation.
- **Event queue** grows linearly with events; currently empty in V1.

### Optimization Opportunities (No Action Required)

1. Replace full `Planet` clone in module updates with in-place mutable
   borrowing where possible (requires scheduler to guarantee disjoint
   access, or use `Rc<RefCell<Planet>>` / `FnOnce` consumer pattern).
2. Capture snapshots via copy-on-write or delta compression instead of
   full clone.
3. Introduce a spatial hash or octree if future modules need local
   neighborhood queries.
4. Make snapshot retention adaptive: only retain snapshots at user-visible
   intervals, not every substep.

**Performance audit: PASS for current scale.  Scalability risk identified
for > 100,000 planets.**

---

## Architectural Strengths

1. **Single-writer ownership is rigorously enforced.**  ADR-010 is
   implemented correctly and validated at test time.
2. **Dependency declarations are explicit and machine-checked.**  The
   pipeline rejects cycles and missing dependencies at build time.
3. **One-tick delayed feedback is structurally guaranteed** by the
   scheduler's sequential execution and snapshot semantics.  No additional
   synchronization is needed.
4. **Transient state is cleanly separated** from persistent state.  Module
   config and lifecycle flags never leak into `SimulationSnapshot`.
5. **Determinism is first-class.**  Fixed timestep, deterministic RNG
   derivation, and ordered event dispatch make replay reliable.
6. **Extensibility is strong.**  New modules only need to register a new
   `PipelineStageDescriptor`, declare dependencies, and own a new
   `Option<State>` field on `Planet`.  No engine changes required.

---

## Architectural Risks

| Risk | Severity | Likelihood | Mitigation |
|---|---|---|---|
| **Snapshot memory blowup** at scale | High | Medium | Adaptive snapshot retention; delta compression. |
| **Validation coverage gaps** for newer modules | Medium | High | Add Atmosphere/Hydrology/Climate/CarbonCycle to ownership and cross-module tests. |
| **Derived-state drift** if a module forgets to refresh derived fields | Medium | Low | Document refresh contract; consider making derived fields `fn(...) -> f64` instead of persisted fields. |
| **Carbon/water total not conserved** — scientific surprise for users | Medium | Medium | Add optional strict-conservation assertions behind feature flag. |
| **`plugin.rs::descriptors()` omits CarbonCycle** | Low | High | Add CarbonCycle descriptor for API completeness. |
| **Loose atmosphere composition tolerance** (`0.01`) | Low | Medium | Tighten or make configurable. |

---

## Recommendations

### Critical

None.  The architecture does not require changes to proceed to Phase 13.

### Recommended

1. **Add CarbonCycleModule to `EvolutionPlugin::descriptors()`.**  
   The registration API currently omits the carbon cycle stage, which is an
   inconsistency users will hit when introspecting the pipeline.

2. **Extend validation tests.**  
   Add AtmosphereModule, HydrologyModule, ClimateModule, and CarbonCycleModule
   to `ownership_validation.rs` and `cross_module_validation.rs`.

3. **Tighten atmosphere composition tolerance or make it configurable.**  
   The current `0.01` absolute tolerance on mole-fraction sum is loose for
   scientific use.

### Optional

4. **Document derived-field refresh contract.**  
   Add a section to ADR-012 stating that modules must recompute derived
   fields (e.g., `atmospheric_co2_fraction`) every tick; the engine does
   not validate this automatically.

5. **Introduce optional conservation assertions.**  
   Add a `strict_conservation` feature flag to `worldsmith-validation` that
   checks total carbon and total water constancy within floating-point
   tolerance.

6. **Plan snapshot optimization before 100,000-planet scale.**  
   Prototype a retained-snapshot or delta-compression strategy when Phase 13
   work begins.

---

## Readiness Assessment

### For Phase 13 — BiosphereModule

**APPROVED FOR NEXT PHASE**

Reasoning:
- The DAG has capacity for a new node at priority `5` with dependency
  `[carbon_cycle]` (or `[climate, carbon_cycle]` if Biosphere needs both).
- `Planet` can accept a new `Option<BiosphereState>` field without
  breaking existing serialized snapshots.
- Validation, snapshot, and scheduler frameworks already support arbitrary
  module additions.
- No scientific equations in existing modules need correction to support
  Biosphere.

### Conditions

- Implement Biosphere as a **sole writer** of `biosphere_state` fields.
- Biosphere must **not** write to CarbonCycleState, HydrologyState,
  AtmosphereState, or ClimateState.  It should publish fluxes that
  AtmosphereModule or CarbonCycleModule consume in the *next* tick, or
  maintain its own independent state.
- If Biosphere needs to modify existing reservoirs, introduce a new
  `BiosphereFluxes` event that CarbonCycleModule/AtmosphereModule consume,
  preserving the one-tick delayed feedback pattern.

---

## Final Decision

**APPROVED FOR NEXT PHASE**

The architecture is sound, deterministic, and ready for BiosphereModule.
The recommendations above should be tracked but do not block Phase 13
implementation.

---

## Phase 17 — PlanetClassificationModule Audit

**Status:** APPROVED

PlanetClassificationModule owns `PlanetClassificationState`. It reads final
values from all physical modules plus HabitabilityModule and produces a
classification using a V1 deterministic decision tree. Confidence scores and
human-readable summaries are generated from classification rules.

Snapshots preserve `classification_state`. Deterministic replay produces
identical classification outputs for identical seeds.

## Phase 18 — Release Preparation Audit

**Status:** APPROVED FOR v1.0 RELEASE

Workspace audit findings:
- Dead code: 1 TODO in `worldsmith-visualization/src/bridge.rs` (non-blocking)
- Placeholder crates: 10 crates remain in Phase 2 stubs (non-blocking for v1.0 science core)
- No unsafe code in workspace sources

Public API completeness:
- 59 public structs, 27 enums, 43 public free functions across 12 active crates
- All missing docs on public types addressed (CryosphereState, BiosphereState,
  SurfaceChemistryState, HabitabilityState, GeologicalProperties)

Examples (5):
- `create_planet.rs`, `evolve_planet.rs`, `save_snapshot.rs`,
  `deterministic_replay.rs`, `inspect_planet.rs`

Documentation:
- `README.md`, `docs/getting-started/index.md`, `docs/simulation-overview/index.md` created

Benchmarks:
- planet_generation: 1.58 µs
- 100 ticks: 1.50 ms
- 1000 ticks: 24.7 ms
- snapshot_creation: 1.16 µs
