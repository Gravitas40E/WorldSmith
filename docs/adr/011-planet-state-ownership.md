# ADR 011: Planet State Ownership

## Status
Proposed — Phase 10A architecture decision.

## Context
Phase 10 introduces a multi-module planetary evolution framework.  Every
scientific discipline (core, mantle, tectonics, atmosphere, climate, ...)
needs to read and write shared planetary data. Without an explicit ownership
model, modules will begin to couple through shared mutable state, breaking
determinism and making save/reload impossible.

The orbital subsystem already solved this problem in Phase 9:

- `OrbitalDynamicsModule` is the **sole runtime writer** to `Planet.position_m`
  and `Moon.velocity_m_s`.
- It reads only `OrbitalProperties` from existing models.
- It communicates changes through deterministic `SimulationEvent`s.
- Consumers (visualization, export) access positions through
  `SimulationSnapshot` only.

Planet evolution needs the same clarity.

## Decision
Establish a **strict single-writer, multiple-reader ownership model** for
all persistent planetary science state.

- **One module = one owner** for each field or field group.
- **WorldState** remains the single mutable source of truth.
- **SimulationSnapshot** is the only view exposed to rendering, export, UI,
  and tooling.
- **Transient solver data** stays inside the producing module; it never enters
  `WorldState` or `Planet`.

### State Classification

Persistent State
	stored in `WorldState` / `Planet` and cloned automatically into every
	`SimulationSnapshot`.

	Examples:
	- `core_temperature`
	- `mantle_temperature`
	- `internal_heat`
	- `tectonic_activity`
	- `volcanic_activity`
	- `volcanic_flux`
	- `magnetic_field_strength`
	- `atmosphere_mass`
	- `atmospheric_composition`
	- `ocean_mass`
	- `ice_mass`
	- `surface_pressure`
	- `surface_temperature`
	- `habitability_index`
	- `life_stage`
	- `ocean_coverage`
	- `cloud_coverage`

Derived State
	computed from persistent state on demand. Not stored in `Planet`.

	Examples:
	- `greenhouse_factor`
	- `escape_velocity`
	- `equilibrium_temperature`
	- `seasonal_variation`
	- `volcanic_risk`
	- `surface_gravity` (already computed from mass/radius but cached in
	  `PhysicalProperties` for compatibility)

	Rule: if a value can be computed deterministically from fields that are
	already stored, do not store it.

Transient State
	module-local scratch memory. Lives only for the duration of `update()`.

	Examples:
	- temporary solver buffers
	- climate iteration caches
	- convergence history
	- numerical work arrays

	Rule: transient state never appears in `WorldState`, `Planet`, or
	`SimulationSnapshot`.

### Ownership Table

| Field / Field Group                      | Owner module                    | Writers | Readers |
|------------------------------------------|---------------------------------|---------|---------|
| `position_m` / `velocity_m_s`            | `OrbitalDynamicsModule`         | 1       | many    |
| `orbit` (elements)                       | Planet formation / stellar      | 1       | many    |
| `PhysicalProperties`                     | Planet formation                | 1       | many    |
| `core_temperature`, `internal_heat`      | `CoreEvolutionModule`           | 1       | many    |
| `mantle_temperature`, lithosphere state  | `MantleEvolutionModule`         | 1       | many    |
| `tectonic_activity`, fault state         | `PlateTectonicsModule`          | 1       | many    |
| `volcanic_activity`, `volcanic_flux`     | `VolcanismModule`               | 1       | many    |
| `AtmosphericProperties`                  | `AtmosphereModule`              | 1       | many    |
| `ocean_mass`, `ice_mass`, `ocean_coverage` | `HydrologyModule`             | 1       | many    |
| `surface_temperature`, climate indices    | `ClimateModule`                 | 1       | many    |
| `magnetic_field_strength`                | `MagneticFieldModule`           | 1       | many    |
| `habitability_rating`, `life_stage`      | `BiosphereModule`               | 1       | many    |
| `EventPayload` variants                  | producing module                | 1       | many    |

### Planet Structure

**Recommended: Option B — Nested composition for new subsystems.**

Current `Planet` already uses composition for several domains:
	`orbit: OrbitalProperties`
	`physical: PhysicalProperties`
	`atmosphere: Option<AtmosphericProperties>`

Keep these existing fields unchanged.  New long-lived science state should
follow the same pattern:

```rust
pub struct Planet {
    // ... existing fields ...
    pub interior: Option<InteriorState>,
    pub climate: Option<ClimateState>,
    pub biosphere: Option<BiosphereState>,
}
```

Advantages:
- Clear ownership: each nested struct has a single responsible module.
- No namespace pollution at the `Planet` level.
- Backward compatible: existing flat fields remain; new groups are optional.
- Future-friendly: adding a new discipline never forces another module to
  watch for name collisions.

Disadvantages:
- Slightly deeper access paths (`planet.climate.surface_temperature_k`).
- Requires one more struct per discipline.

Option A (flat fields on `Planet`) is rejected because it worked for the
initial model but does not scale to 10+ evolving properties.

For fields that already exist flat (e.g., `tectonic_activity`, `volcanic_activity`,
`magnetic_field_strength`, `surface_temperature_k`, `ocean_coverage`, `ice_mass`),
**keep them flat until a Phase 10B+ migration when the owning module is
implemented**.  At that point the module may optionally wrap them in a nested
struct if the field count justifies it.

### Module Read/Write Matrix

| Module                     | Reads (FieldKey)                              | Writes (FieldKey)                               | Never modifies            |
|----------------------------|-----------------------------------------------|-------------------------------------------------|---------------------------|
| OrbitalDynamicsModule      | OrbitalElements, PlanetMass, StarPosition     | none in current refactor; writes `position_m`/`velocity_m_s` directly | model composition         |
| CoreEvolutionModule        | PlanetMass                                    | InteriorState / core_temperature (future)       | orbit, atmosphere         |
| MantleEvolutionModule      | PlanetMass, CoreHeatFlux (future)             | InteriorState / mantle_temperature (future)     | orbit, hydrosphere        |
| PlateTectonicsModule       | MantleTemperature, InteriorHeat (future)      | TectonicActivity, VolcanicFlux (future)         | atmosphere, ocean         |
| VolcanismModule            | TectonicActivity, MantleTemperature (future)  | VolcanicActivity, AtmosphericComposition (future) | hydrosphere, climate      |
| AtmosphereModule           | VolcanicFlux, OceanMass (future)              | AtmosphericProperties                           | interior, orbit           |
| HydrologyModule            | SurfaceTemperature, AtmosphereMass (future)   | OceanMass, IceMass, OceanCoverage               | interior, tectonics       |
| ClimateModule              | AtmosphereMass, OceanMass, Albedo (future)    | SurfaceTemperature, ClimateIndices (future)     | interior, tectonics       |
| MagneticFieldModule        | CoreTemperature, RotationPeriod (future)      | MagneticFieldStrength                           | atmosphere, hydrosphere   |
| BiosphereModule            | Climate, Ocean, Atmosphere (future)           | HabitabilityRating, LifeStage                   | interior, tectonics       |

Note on FieldKey growth: New `FieldKey` variants must be added to
`worldsmith-state` when new field groups are introduced.  Existing modules
continue to compile because they consume only the keys they declare.

### Events

Events are the **observability and decoupling mechanism**, not the primary
state-mutation path.

Rule:
	Module -> writes state directly into WorldState -> emits typed event.

Why:
- Direct writes keep `SimulationSnapshot` consistent without a second
  reconciliation pass.
- Events let unrelated modules react without declaring a hard dependency on
  every module's API.
- Events are deterministic (ordered by `timestamp_s` and `EventId`) and
  therefore replay-safe.

Events to model in Phase 10:

- `CoreFormed { planet_id }`
- `PlanetDifferentiated { planet_id }`
- `VolcanicEruption { body, intensity }`
- `AtmosphereCreated { planet_id }`
- `AtmosphereCollapsed { planet_id }`
- `OceanFormed { planet_id }`
- `ClimateShift { planet_id }`
- `MagneticFieldReversed { planet_id }`
- `HabitabilityChanged { planet_id }`
- `BiosphereEmerging { planet_id }`

Snapshots must capture enough state to reconstruct the event history from
deterministic replay; therefore **events must be stored deterministically**
but need not be stored in `Planet` itself.

### Determinism

To preserve determinism across modules:

1. All modules receive the same absolute `ModuleContext.timestamp_s`.
2. All modules must read from `WorldState` and write to `WorldState` only.
3. No module may access global RNG directly; use `Engine::rng_stream(label)`.
4. Writers must not observe intermediate states of other writers in the same
   tick. The scheduler executes modules sequentially, so reads captured at
   the start of a tick remain valid for that tick.
5. Events emitted during `update()` are pushed into `WorldState.event_queue`
   and consumed only by modules later in the pipeline.

### Serialization

Strategy:
	Versioned struct serialization via serde.

- Every struct that crosses a save/load boundary (`WorldState`, `Planet`,
  nested state structs, `SimulationSnapshot`) derives `Serialize` +
  `Deserialize`.
- `SimulationMetadata.schema_version` is the global schema counter.
- New fields are added with `Option<...>` and default values so older saves
  deserialize.
- Removed fields are simply dropped; no migration complexity because old data
  is optional.
- No custom binary format in Phase 10A; JSON is acceptable for tooling and
  tests.

Snapshots:
- `SimulationSnapshot` clones the full authoritative state.
- `PlanetSnapshot` wraps the complete `Planet`, including all nested state.
- Rendering and tools must never request partial snapshots; requesting the
  full snapshot keeps the contract simple and avoids version drift.

### Snapshot Strategy

Current shape:
	SimulationSnapshot {
		stellar: StellarSnapshot,
		planets: Vec<PlanetSnapshot>,
		moons: Vec<Moon>,
	}

Contain everything persistent. Rationale:
- Planet evolution data is small compared to raw geometry or texture memory.
- Missing fields in snapshots create hidden coupling between modules and
  consumers.
- If snapshot size becomes a bottleneck at 10k–100k planets, optimization is
  compression or differential snapshots, not field deletion.

`VisualSnapshot`:
- Keep as a separate viewport-optimized layout if rendering needs fewer
  fields, but derive it from `SimulationSnapshot` after capture, not from
  `WorldState` directly.

### Performance & Memory

| Scale        | Estimated Planet memory (w/ nested state) | Notes                                              |
|--------------|--------------------------------------------|----------------------------------------------------|
| 100          | ~32 KB                                     | Negligible; CPU cache friendly.                     |
| 1,000        | ~320 KB                                    | Small; fits L2 cache on modern CPUs.               |
| 10,000       | ~3.2 MB                                    | L3 cache resident on consumer CPUs.                |
| 100,000      | ~32 MB                                     | RAM resident; serialization dominates.             |

Parallelism:
- `WorldState` maps are `BTreeMap`, which is not parallel-friendly.
- Future optimization: switch to `FxHashMap` or `HashMap` for internal maps,
  or batch `Planet` updates into parallel arrays by discipline.
- Modules themselves can parallelize across planets using `rayon` as long as
  they serialize writes back to `WorldState` in deterministic order.

GPU / ECS compatibility:
- Nested state structs map cleanly to ECS components (one component per struct
  per body).
- Avoid storing raw `Vec<f64>` buffers in `Planet`; keep arrays inside
  module-local state during computation and write only scalar summaries.

### Extensibility

Future systems must integrate without modifying existing evolution modules.

Design rules:
1. New discipline = new `SimulationModule` + nested state struct (optional).
2. New module declares its reads/writes through `FieldKey` and emits events.
3. Existing modules do not need to acknowledge new fields unless they depend
   on them.
4. Non-scientific systems (Life, Civilizations, Terraforming, Colonies,
   Megastructures, Mining, Planetary Engineering) are implemented as **event
   consumers** or independent `SimulationModule`s that observe state rather
   than replace it.
5. State added by future systems lives in `WorldState` maps or nested state,
   never in ad-hoc global registries.

### Module Contracts

Every module must respect:
- **Read-only access** through `StateReader`.
- **Mutable access** through `StateWriter` only for the fields it owns.
- **No hidden side effects**: no RNG, no file IO, no networking.
- **Deterministic output**: given identical `timestamp_s`, `seed`, and
  `WorldState`, the module must produce identical mutations and events.

### Recommended Implementation Order

1. **CoreEvolutionModule** — establish nested state write path.
2. **MantleEvolutionModule** — add dependency on core output.
3. **AtmosphereModule placeholder** — validate event architecture.
4. **Hydrology / Climate placeholders** — extend FieldKey vocabulary.
5. **Snapshot and save/load validation** — confirm versioned serialization.

### Major Architectural Risks

- **Accidental shared mutation**: mitigations are code review + eventual
  scheduler-level read/write contract enforcement.
- **Snapshot bloat at scale**: mitigated by compression and differential
  snapshots if needed later.
- **ECSS migration friction**: mitigated by keeping nested state structs small
  and component-shaped from inception.
- **Determinism drift from floating-point non-associativity**: mitigated by
  reproducible summation order (BTreeMap iteration) and identical module
  ordering.

### Confidence Level

High. The ownership model mirrors the already-proven orbital pattern. The
main implementation risk is migrant fields from existing `Planet` flat
storage into nested structs, which can be performed incrementally one field
group at a time.