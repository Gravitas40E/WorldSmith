# Phase 9 Design: Orbital Dynamics

## Status
Proposed — awaiting implementation

## Context

The current simulation layer stores orbital elements on `Planet` and `Moon`
but never converts them into world-space coordinates. `Star` and
`StellarSystem` carry `position_m` / `velocity_m_s`, yet they remain at
`Vector3::ZERO` because no module updates them. The visualization layer
presently hardcodes planet/moon positions to `[0.0, 0.0, 0.0]` with TODO
comments, which makes physically correct rendering impossible.

`worldsmith-math::orbital` already contains `circular_orbit_state` and
`elliptical_orbit_state`, but no simulation module invokes them. The engine
pipeline captures a `SimulationSnapshot` after every fixed timestep, but the
snapshot only reflects what modules have written into `WorldState`.

The goal is to compute deterministic, physically correct world-space positions
for stars, planets, and moons inside the simulation layer so that every
downstream consumer (visualization, export, replay, climate insolation) reads
the same coordinate from the same source of truth.

---

## 1. Existing simulation pipeline review

**Runtime path:**
- `Engine::tick()` accumulates real time into a fixed-timestep accumulator.
- For each fixed step it calls `Scheduler::step(state, registry, pipeline, delta)`.
- `Scheduler::step` iterates `pipeline.execution_order()`, builds a
  `ModuleContext { timestamp_s, delta_seconds, seed }` for each module,
  calls `module.update(context, state)`, collects published events, then
  dispatches events to all modules.
- After the scheduler finishes, `Engine::capture_snapshot()` pushes
  `WorldState::snapshot()` if rendering is enabled.

**Pipeline ordering today (from `docs/design/pipeline-stages.md`):**
1. `preset_load`
2. `stellar_formation`
3. `planetary_accretion`
4. `geology_bulk`
5. `atmosphere_initial`
6. `climate_initial`
7. `geology_surface`
8. `snapshot_emit`

Modules that can change orbital elements after generation:
- `PlanetEvolutionModule` can modify `planet.orbit.semi_major_axis_m`.
- Future tidal/inclination modules will write orbital parameters.

Because the snapshot is taken **after** all scheduled updates, any module
that runs before snapshot capture can update positions and those positions
will appear in the snapshot automatically.

---

## 2. Where `OrbitalDynamicsModule` should execute

**Decision:** Implement it as a **runtime `SimulationModule`**, not a
generation-time `PipelineStage`.

**Rationale:**
- Generation-time stages run once during creation. Orbital propagation must
  continue across every fixed tick during runtime.
- The existing scheduler already provides deterministic ordering, delta time,
  and event dispatch.
- A `SimulationModule` can read/write `WorldState` and participate in the
  same field-registry contract.

**Registration / ordering:**
- Register with `EngineBuilder::register_module_with_stage(...)`.
- Priority should be **highest among simulation modules** so it runs last.
  This ensures it sees any orbital-element changes written by earlier modules
  in the same tick (e.g. `PlanetEvolutionModule` updating `semi_major_axis_m`).
- Dependencies: should declare dependency on `planetary_accretion` and any
  evolution/cosmology modules that write orbital fields. For a minimal
  implementation, declare dependency on `worldsmith.planet_formation` and
  `worldsmith.planet_evolution` if present.
- If no orbital-element writer is registered, the module should still run and
  recompute positions from the last known elements.

**Effect on existing pipeline:**
- No existing module or stage needs to move.
- The orbital module simply adds itself to the ordered module list.

---

## 3. Inputs

The module reads the following from `WorldState` via `StateReader`:

| Source | Data | Purpose |
|--------|------|---------|
| `Star` | `position_m`, `mass_kg.value`, `id` | Parent reference for planets |
| `StellarSystem` | `position_m` | Barycenter reference |
| `Planet` | `orbit.parent`, `orbit.semi_major_axis_m.value`, `orbit.eccentricity.value`, `orbit.inclination_rad.value`, `orbit.orbital_period_s`, `physical.mass_kg.value` | Orbital elements & mass |
| `Moon` | `orbit.parent`, `orbit.semi_major_axis_m.value`, `orbit.eccentricity.value`, `orbit.inclination_rad.value`, `orbit.orbital_period_s`, `physical.mass_kg.value` | Orbital elements & mass |
| `ModuleContext` | `timestamp_s`, `delta_seconds`, `seed` | Propagation time & determinism |

The module also uses `worldsmith-math::orbital` for:
- Kepler period from mass and semi-major axis (when `orbital_period_s` is `None`)
- Mean anomaly from time
- Eccentric anomaly solver
- Radius and velocity in the orbital plane
- Inclination rotation matrix

The module does **not** read `VisualSnapshot`, render state, or UI state.

---

## 4. Outputs

The module writes the following into `WorldState` via `StateWriter`:

| Target | Fields written | Meaning |
|--------|---------------|---------|
| `Planet` | `position_m: Vector3`, `velocity_m_s: Vector3` | Barycentric world-space state |
| `Moon` | `position_m: Vector3`, `velocity_m_s: Vector3` | Barycentric world-space state |
| `Star` | `position_m: Vector3`, `velocity_m_s: Vector3` | System-relative state (for binaries / multi-star) |

**Events published:**
- `EventPayload::OrbitalChanged { target: EventTarget::Planet(planet_id) }`
- `EventPayload::OrbitalChanged { target: EventTarget::Moon(moon_id) }`
- Optionally `EventPayload::OrbitalChanged { target: EventTarget::Star(star_id) }`

Downstream modules (climate, visualization bridge) can subscribe to these
events rather than polling all bodies every tick.

**No new fields are added to `SimulationSnapshot`.** Because `WorldState::snapshot()`
clones `Planet` and `Moon` structs directly, the new position/velocity fields
appear in every snapshot once they exist on the model types.

---

## 5. Parent-child hierarchy

The hierarchy is encoded in `BodyReference` and the existing `Planet.orbit.parent`
and `Moon.parent` fields.

```
StellarSystem (barycenter)
├── Star(s)
│   └── Planet(s)
│       └── Moon(s)
│           └── Moon(s)  // nested, supported by BodyReference::Moon
```

**Position resolution rules:**
1. **System barycenter** (`StellarSystem.position_m`): fixed at origin unless a
   future multi-star dynamics module moves it. Single-star systems default to
   `Vector3::ZERO`.
2. **Star** (`Star.position_m`): for single-star systems, equals
   `StellarSystem.position_m`. For binaries, offsets relative to barycenter
   via mutual orbit (future extension).
3. **Planet**: absolute position = `parent_world_position` + `orbital_offset`.
   - `parent_world_position` is looked up from the referenced `Star` or `Planet`
     according to `Planet.orbit.parent`.
4. **Moon**: absolute position = `parent_world_position` + `orbital_offset`.
   - `parent_world_position` is the `Planet.position_m` resolved from
     `Moon.orbit.parent`.
5. **Nested moons**: if `Moon.parent == BodyReference::Moon(child_id)`, the
   module resolves the ancestor chain recursively until it reaches a `Star` or
   `StellarSystem`.

**Cycle prevention:**
- `BodyReference` is an enum, not a graph pointer, so cycles cannot be formed
  at the type level. The module resolves parents via BTreeMap lookups by ID.
- If a parent ID is missing from `WorldState`, the module leaves the child at
  `Vector3::ZERO` and publishes `OrbitalChanged` so diagnostics can detect
  the inconsistency.

**Mass lookup for gravity:**
- Planet/moon orbit central mass is the parent body's `physical.mass_kg.value`.
- For planets orbiting stars, use the referenced `Star.mass_kg.value`.
- For moons orbiting planets, use the referenced `Planet.physical.mass_kg.value`.

---

## 6. Timestamp-driven orbital propagation

**Deterministic time source:**
- The scheduler passes `ModuleContext { timestamp_s, delta_seconds, seed }`.
- `timestamp_s` is the **absolute simulation time** at the start of the fixed
  step. It is monotonically increasing, identical across all modules in the
  same tick, and reproducible for the same seed and input parameters.
- `delta_seconds` is the fixed timestep duration (e.g. 1/60 s).

**Propagation method:**
- For Keplerian orbits (the current model assumption), position is a pure
  function of `timestamp_s`, not an integration over `delta_seconds`.
- The module computes mean anomaly at epoch `t`:
  ```
  M(t) = 2π * (t / T)  (mod 2π)
  ```
  where `T = orbital_period_s.unwrap_or_else(|| kepler_period(parent_mass, a))`.
- If `orbital_period_s` is `None`, derive it once from `kepler_period` and
  cache it back into `Planet.orbit.orbital_period_s` so subsequent ticks do
  not recompute it. This preserves determinism and improves performance.
- Solve Kepler's equation `M = E - e * sin(E)` for eccentric anomaly `E`
  using Newton-Raphson (3-5 iterations is sufficient for visual/eclipse
  accuracy).
- Convert `E` to true anomaly `ν`.
- Compute orbital radius:
  ```
  r = a * (1 - e²) / (1 + e * cos(ν))
  ```
- Build position in orbital plane (Z=0):
  ```
  x = r * cos(ν)
  y = r * sin(ν)
  ```
- Rotate by `inclination_rad` about the X axis:
  ```
  X = x
  Y = y * cos(i)
  Z = y * sin(i)
  ```
- Offset by parent world position and add to `Planet.position_m`.

**Why absolute timestamp, not incremental delta:**
- Absolute time avoids accumulation-of-rounding error across thousands of ticks.
- It makes replay trivial: given the same initial state and the same
  `timestamp_s`, every body returns to exactly the same position.
- It allows seeking / scrubbing without forward-integration.
- `delta_seconds` is retained for modules that need rate-of-change, but the
  orbital module does not use it for position computation.

**Velocity:**
- Computed from orbital mechanics (vis-viva + rotation of orbital-plane
  velocity vector) so that downstream momentum-dependent modules have a
  deterministic value.

---

## 7. Integrating `worldsmith-math::orbital`

Current orbital functions:
- `circular_orbit_state(central_mass, radius, angle)` → `OrbitState`
- `elliptical_orbit_state(central_mass, sma, ecc, true_anomaly)` → `OrbitState`
- `kepler_period(central_mass, sma)` → period
- `elliptical_radius(sma, ecc, true_anomaly)` → radius

**New helpers to add to `worldsmith-math/src/orbital.rs`:**

1. **`mean_anomaly_from_time(timestamp_s: f64, period_s: f64) -> f64`**
   - Returns `(2π * timestamp_s / period_s) mod 2π`.
   - Pure math, no error cases.

2. **`eccentric_anomaly_from_mean(mean_anomaly: f64, eccentricity: f64) -> f64`**
   - Newton-Raphson on `E - e sin(E) - M = 0`.
   - Initial guess: `E0 = M` for `e < 0.8`, else `E0 = π`.
   - Converge to double-precision tolerance `1e-12` or 20 iterations max.
   - Returns `f64`, never errors.

3. **`true_anomaly_from_eccentric(eccentricity: f64, eccentric_anomaly: f64) -> f64`**
   - `ν = 2 * atan2(√(1+e) * sin(E/2), √(1-e) * cos(E/2))`
   - More stable than `cosν / sinν` formulas near apocenter.

4. **`propagate_orbit_state(central_mass_kg: f64, sma_m: f64, eccentricity: f64, inclination_rad: f64, period_s: f64, timestamp_s: f64) -> OrbitState`**
   - Orchestrates the above: mean anomaly → eccentric anomaly → true anomaly →
     radius → orbital-plane position → inclination rotation.
   - Returns position and velocity in the **parent-relative** frame.

**What stays unchanged:**
- Existing `circular_orbit_state` and `elliptical_orbit_state` remain public
  and are used by tests.
- `OrbitState` struct stays as-is.

**Why math lives in `worldsmith-math`:**
- It is pure function mathematics. Any crate can use it for validation,
  visualization, or export without pulling in simulation state.
- Keeps the OrbitalDynamicsModule focused on state access and scheduling,
  not on equation derivation.

---

## 8. Why visualization must not perform orbital projection

**Rule:** `worldsmith-visualization` consumes `SimulationSnapshot` only. It
must not reconstruct world-space positions from orbital elements.

**Reasons:**

1. **Single source of truth.** If both the simulation module and the
   visualization bridge compute positions, they will diverge due to different
   implementations, tolerance choices, or frame-of-reference bugs. Downstream
   consumers must never choose between two positions for the same body.

2. **Determinism and replay.** Replay, golden tests, and save-file
   reproduction rely on identical position sequences for identical inputs.
   If position computation lives outside the engine, external code cannot
   reproduce the exact same floating-point trajectory.

3. **Cross-module dependencies.** Climate insolation, magnetic field
   interaction, and collision detection all need absolute positions. They
   must read one authoritative `position_m` from `WorldState`, not each
   re-implement Kepler propagation.

4. **Architecture boundary.** `ARCHITECTURE.md` is explicit:
   - **visualization** knows models and colormaps, not wgpu or physics.
   - **render** knows GPU, not solvers.
   - **simulation** owns physics and state mutation.
   Putting orbital mechanics in visualization violates all three layers.

5. **Snapshot contract.** `SimulationSnapshot` is already the published
   boundary. Adding `position_m`/`velocity_m_s` to `Planet`/`Moon` means
   every snapshot automatically carries正确答案 coordinates. The bridge
   merely reads them, converting `[f64; 3]` to render-space as needed.

**Current bridge violation:**
The `DefaultSnapshotBridge` already hardcodes `[0.0, 0.0, 0.0]` for planets
and moons. This is a known placeholder. After Phase 9, the bridge should
read `planet.position_m` and `moon.position_m` directly. No projection logic
should enter the visualization crate.

---

## 9. Minimal model changes

**Affected file:** `worldsmith-models/src/lib.rs`

**Changes:**
- Add `position_m: Vector3` and `velocity_m_s: Vector3` to `Planet`.
- Add `position_m: Vector3` and `velocity_m_s: Vector3` to `Moon`.
- Keep `#[derive(Default)]` or manually default to `Vector3::ZERO` so existing
  `PlanetFormationModule::build_from_state` and tests compile unchanged.
- Do **not** modify `Star` or `StellarSystem` position fields; they already
  exist but default to ZERO via builder defaults.

**Why this is safe:**
- `WorldState::planets` is a `BTreeMap<PlanetId, Planet>`. Adding fields to
  `Planet` does not affect map operations.
- `PlanetSnapshot` wraps `Planet` via `From<Planet>`; it automatically
  includes new fields.
- `SimulationSnapshot::planets: Vec<PlanetSnapshot>` likewise carries them.
- Existing serialization tests for `Planet` gain two extra fields with default
  values; serde tolerates this for non-tag-based formats.

---

## 10. Implementation plan

### Phase A — Math extension (`worldsmith-math`)
- Add `mean_anomaly_from_time`, `eccentric_anomaly_from_mean`,
  `true_anomaly_from_eccentric`, `propagate_orbit_state` to `src/orbital.rs`.
- Add unit tests covering:
  - Circular orbit at 0, π/2, π, 3π/2 matches `circular_orbit_state`
  - Elliptical orbit mean-anomaly round-trip accuracy
  - `propagate_orbit_state` at `t=0` and `t=T` returns same state

### Phase B — Model extension (`worldsmith-models`)
- Add `position_m` and `velocity_m_s` to `Planet` and `Moon` with ZERO
  defaults.
- Run `cargo check --workspace` to confirm no downstream breakage.

### Phase C — OrbitalDynamicsModule (`worldsmith-stellar`)
- Create `src/orbital_module.rs`.
- Implement `SimulationModule` + `PipelineStage`:
  - `id()`: `"worldsmith.orbital_dynamics"`
  - `initialize()`: pre-derive `orbital_period_s` for every planet/moon that
    lacks it, using parent mass and `kepler_period`. This avoids per-tick
    branches on `None`.
  - `update()`:
    1. Iterate `state.world().planets` and `state.world().moons`.
    2. Resolve parent world position via `BodyReference`.
    3. Call `worldsmith_math::orbital::propagate_orbit_state` with
       `context.timestamp_s`.
    4. Write `position_m` and `velocity_m_s` back into `WorldState`.
    5. Publish `OrbitalChanged` events only when position changes by more
       than a small epsilon (reduces downstream event-processing load).
  - `reads()`: `FieldKey::OrbitalElements`
  - `writes()`: no new FieldKey needed; the module mutates model fields
    directly. Optionally add a new `FieldKey::WorldPosition` in future if
    the field-registry contract becomes stricter.
- Determinism guarantee: no RNG, no interior mutability, all inputs from
  immutable module context or already-committed `WorldState` fields.

### Phase D — Registration and wiring
- Update `worldsmith-stellar/examples` / integration tests to register the
  module with **priority 30** and dependencies on `worldsmith.planet_formation`
  and `worldsmith.planet_evolution` (if those modules are present in the run).
- Update `worldsmith-engine/examples/sandbox.rs` with a commented example.
- Add a regression test in `worldsmith-stellar/tests/orbital.rs` that:
  1. Builds an engine with StellarModule + PlanetFormationModule +
     OrbitalDynamicsModule.
  2. Runs `tick_fixed()` multiple times.
  3. Asserts `planet.position_m` is non-zero after the first tick.
  4. Asserts two identical engine runs produce identical position sequences.

### Phase E — Visualization bridge cleanup (`worldsmith-visualization`)
- In `src/bridge.rs`, replace the `position_m: [0.0, 0.0, 0.0]` placeholders
  for planets and moons with:
  ```
  position_m: [planet.position_m.x, planet.position_m.y, planet.position_m.z]
  ```
  and similarly for moons.
- Remove the TODO comments.
- Stars already read `star.position_m`; no change needed there.

### Phase F — Documentation
- Update `docs/design/pipeline-stages.md` to list `orbital_dynamics` as a
  runtime module with priority/configurable dependencies.
- Add a new ADR (this document) under `docs/adr/`.
- Update `ARCHITECTURE.md` "Future Scalability" section to mark orbital
  dynamics as implemented rather than future.

---

## 11. Alternatives considered and rejected

**A. Compute positions in the visualization bridge only**
- Rejected: violates the physics-owns-state rule; every consumer would need
  its own projection; determinism and replay break.

**B. Add positions during `PlanetFormationModule::initialize` only**
- Rejected: positions would be static. No propagation over time. Not useful
  for animation or time-varying insolation.

**C. Use N-body numerical integration (Runge-Kutta)**
- Rejected for Phase 9: overkill, slower, harder to determinize across
  platforms. Keplerian propagation is exact for two-body and sufficient for
  the current planet-formation abstraction. N-body can replace this module
  later without changing its interface.

**D. Put the module in a new `worldsmith-orbital` crate**
- Rejected for minimalism: one file in `worldsmith-stellar` is smaller,
  reduces new dependencies, and matches the documented intent
  ("N-body orbital submodule in worldsmith-stellar").

---

## 12. Risks and mitigations

| Risk | Mitigation |
|------|-----------|
| Missing `orbital_period_s` causes division-by-zero or missing period | Pre-derive in `initialize()` from `kepler_period`. Treat `semi_major_axis_m <= 0` as invalid input and publish an error event. |
| Parent body missing from `WorldState` | Leave child at ZERO, publish `OrbitalChanged` event, log via module diagnostics. Do not panic. |
| Circular dependency between stellar and planet crates | `OrbitalDynamicsModule` lives in `worldsmith-stellar`. It reads `Planet`/`Moon` model data (already public in `worldsmith-models`) but does not depend on `worldsmith-planet`. No crate cycle. |
| Determinism across platforms | Use only IEEE 754 double-precision math; no `sin`/`cos` with varying precision. Newton-Raphson iterations are bounded. |
| Performance with thousands of bodies | O(B) per tick is acceptable. Event publishing only on change. Later: spatial hash for parent lookups if needed. |

---

## Summary

The simulation layer has all the inputs needed for physically correct positions
except `Planet.position_m`/`velocity_m_s` and `Moon.position_m`/`velocity_m_s`.
The smallest architectural change is:
1. Add those two fields to the models.
2. Add four math helpers to `worldsmith-math::orbital`.
3. Introduce a single `SimulationModule` in `worldsmith-stellar` that runs
   every tick, propagates orbits from `timestamp_s`, and writes back positions.
4. Update the visualization bridge to read the new fields.

No changes are required to `Engine`, `WorldState`, `SimulationSnapshot`,
`SimulationModule` trait, or the scheduler. The snapshot mechanism already
picks up everything written into `WorldState`.
