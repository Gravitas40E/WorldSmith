# Orbital Dynamics Test Plan

## Scope
Design-only test specification for `OrbitalDynamicsModule` per ADR-009.
No implementation, no code modification.

---

## 1. Test Philosophy

### Strategy
Orbital mechanics is a numerical, state-mutating subsystem inside a
deterministic simulation engine. Tests must therefore validate four distinct
properties: mathematical correctness, deterministic replay, physical fidelity,
and stable integration across the engine-to-visualization boundary.

The suite is split by category so failures map cleanly to a fault domain.

### Categories

| Category | Exists because | When run |
|----------|---------------|---------|
| **Unit tests** | `worldsmith-math::orbital` is pure math and must be provably correct in isolation. | Per-commit; every CI run. |
| **Integration tests** | The module interacts with `WorldState`, `ModuleContext`, `ModuleRegistry`, and `Pipeline`. A contract break in any of those surfaces only at integration time. | Per-commit; every CI run. |
| **Regression tests** | Orbital behavior must remain bit-identical across code churn (refactors, dependency updates, format changes). | Every CI run, ideally with golden files. |
| **Determinism tests** | The simulation promises that identical seed + input → identical trajectory. Cross-step, replay, and save/load consistency must all hold. | Nightly and on engine PRs touching scheduler, clock, or state mutation. |
| **Performance tests** | O(B) per tick is the target; users may run thousands of bodies. We need early warning when orbital update time regresses. | On merge to `main` and before release tags; not required every commit. |

### Why each exists
- **Unit**: catches math regressions fast, no engine setup needed.
- **Integration**: catches model/state/registry contract breaks.
- **Regression**: catches silent float-ordering or reference-frame drift.
- **Determinism**: catches nondeterminism introduced by RNG, unordered map iteration, or platform math differences.
- **Performance**: catches algorithmic regressions before users hit them on large systems.

---

## 2. Math Validation Tests

Location: `worldsmith-math/src/orbital.rs` inline `#[cfg(test)]`.

### 2.1 Mean anomaly from time
- **Purpose:** Verify that propagation starts from a canonical phase.
- **Input:** `timestamp_s = T/2` (half-period), `period_s = T`.
- **Expected:** Mean anomaly = π (mod 2π).
- **Pass criteria:** Returns value in `[0, 2π)` and `abs(result - π) < 1e-12`.
- **Edge checked:** `timestamp_s = 0`, `timestamp_s = T`, `timestamp_s = 100T` — all reduce deterministically.

### 2.2 Mean anomaly from time — negative / large timestamps
- **Purpose:** Ensure reduction works for arbitrary timestamps.
- **Input:** `timestamp_s = -1.5 * T`, `period_s = T`.
- **Expected:** Mean anomaly = `(2π * (-1.5)) mod 2π` = `π`.
- **Pass criteria:** Result matches positive-reference case within tolerance.

### 2.3 Eccentric anomaly solver — circular orbit
- **Purpose:** For `e = 0`, eccentric anomaly equals mean anomaly exactly.
- **Input:** `mean_anomaly = 1.2345`, `eccentricity = 0.0`.
- **Expected:** `abs(result - mean_anomaly) < 1e-12`.
- **Pass criteria:** Converges in ≤ 5 iterations; no Newton divergence.

### 2.4 Eccentric anomaly solver — elliptical orbit
- **Purpose:** Verify Newton-Raphson converges for representative eccentricities.
- **Inputs:** `(M, e)` pairs:
  - `(0.5, 0.1)`
  - `(π, 0.3)`
  - `(4.0, 0.6)`
  - `(5.5, 0.9)`
- **Expected:** Residual `|E - e*sin(E) - M| < 1e-12` or 20 iterations hit.
- **Pass criteria:** All cases converge; high-eccentricity case does not diverge.

### 2.5 Eccentric anomaly solver — boundary conditions
- **Purpose:** Ensure stability at `M = 0`, `M = π`, `M = 2π` and `e → 1`.
- **Input:** `M = 0, e = 0.99`; `M = π, e = 0.99`.
- **Expected:** Finite result, residual < 1e-12.
- **Pass criteria:** No NaN, no infinity.

### 2.6 True anomaly from eccentric — circular
- **Purpose:** When `e = 0`, true anomaly equals eccentric anomaly.
- **Input:** `eccentricity = 0.0`, `eccentric_anomaly = 1.2345`.
- **Expected:** `abs(result - 1.2345) < 1e-12`.
- **Pass criteria:** Matches input exactly.

### 2.7 True anomaly from eccentric — elliptical
- **Purpose:** Verify conversion across full range.
- **Input:** `eccentricity = 0.5`, `eccentric_anomaly` sampled at `0, π/2, π, 3π/2`.
- **Expected:** No discontinuity at `π`/`2π`; values stable.
- **Pass criteria:** No sign-flip artifacts; derivative continuity visible on dense sweep.

### 2.8 Circular orbit propagation
- **Purpose:** Verify `propagate_orbit_state` matches `circular_orbit_state` at the same angle.
- **Input:** `central_mass_kg = 1e30`, `sma_m = 1.496e11`, `eccentricity = 0.0`, `inclination_rad = 0.0`, `period_s = kepler_period(1e30, 1.496e11)`, `timestamp_s = 0.3 * period_s`.
- **Expected:** Position magnitude ≈ `sma_m`; direction consistent with `circular_orbit_state` within 1e-6 relative.
- **Pass criteria:** Radial error < 1 mm at 1 AU; agreement with `circular_orbit_state` within 1e-9 relative.

### 2.9 Elliptical orbit propagation
- **Purpose:** Verify end-to-end elliptical propagation against synthetic state.
- **Input:** `central_mass_kg`, `sma_m = 2 AU`, `eccentricity = 0.3`, `inclination_rad = 0.4`, `period_s`, `timestamp_s ∈ {0, 0.25T, 0.5T, 0.75T, T}`.
- **Expected:**
  - At `t=0` and `t=T`: position identical (within numerical precision).
  - At `t=0.5T`: body roughly opposite the focus from `t=0`.
- **Pass criteria:** Position at `t=T` matches `t=0` within 1e-6 m; radial distance at apoapsis equals `a(1+e)` within 1e-6 m.

### 2.10 Inclination rotation
- **Purpose:** Inclination must tilt the orbital plane without distorting the orbit shape.
- **Input:** `inclination_rad = 0.5`; compare propagated Z component and in-plane magnitude to `inclination = 0`.
- **Expected:** In-plane radius magnitude unchanged; Z is `r * sin(ν) * sin(i)`.
- **Pass criteria:** `sqrt(x² + y²)` equals zero-inclination radius within tolerance; Z sign follows inclination sign.

### 2.11 Velocity magnitude (vis-viva)
- **Purpose:** Ensure returned velocity obeys energy conservation.
- **Input:** Same elliptical inputs as 2.9 at several timestamps.
- **Expected:** `0.5 * v² - G*M / |r|` is constant across the orbit within tolerance.
- **Pass criteria:** Specific orbital energy variation < 1e-3 J/kg across full orbit.

### 2.12 Numerical stability — extreme semi-major axis
- **Purpose:** Ensure propagation does not overflow or denormalize at extremes.
- **Input:** `sma_m = 1e18` (stellar-system scale), `eccentricity = 0.01`, `period_s` computed, `timestamp_s` large.
- **Expected:** Finite position and velocity; no NaN, no Inf.
- **Pass criteria:** Both position and velocity are finite; relative motion still traces a coherent ellipse.

### 2.13 Numerical stability — near-circular with high precision demand
- **Purpose:** Newton-Raphson initial guess must remain stable for `e → 0`.
- **Input:** `eccentricity = 1e-8`, several timestamps.
- **Expected:** Converges without oscillation; eccentric anomaly ≈ mean anomaly.
- **Pass criteria:** Position error < 1e-6 m.

---

## 3. OrbitalDynamicsModule Tests

Location: `worldsmith-stellar/tests/orbital_module.rs`

### 3.1 Planet position updates every tick
- **Initial state:** One `Star` with nonzero mass and position (0,0,0). One `Planet` with valid orbital elements (`a = 1 AU`, `e = 0.0`, `i = 0`), `parent = Star`.
- **Steps:** Call `module.update(context, state)` once.
- **Expected:** `planet.position_m` is non-zero and lies roughly 1 AU from the star; `planet.velocity_m_s` is non-zero.

### 3.2 Moon follows moving parent
- **Initial state:** Same as 3.1 plus one `Moon` whose `parent` is the `Planet`. Moon orbit `a = 384_400_000 m`, `e = 0.0`.
- **Steps:** Run 10 ticks.
- **Expected:** Moon world-space position orbits around planet position at the correct radius. Moon distance to star traces a small epicycle centered near planet's solar orbit.

### 3.3 Parent-child hierarchy resolution — star parent
- **Initial state:** Star `S1` at origin, Planet `P1` with `parent = Star(S1)`.
- **Steps:** One tick.
- **Expected:** `P1.position_m` offset from `Star.position_m` by orbital vector.

### 3.4 Parent-child hierarchy resolution — planet parent
- **Initial state:** Star `S1` at origin, Planet `P1` offset by `[1e6, 0, 0]`. Moon `M1` with `parent = Planet(P1)` and orbit `a = 1e5 m`.
- **Steps:** One tick.
- **Expected:** `M1.position_m` offset from `P1.position_m`, not from star.

### 3.5 Multiple planets around one star
- **Initial state:** One star, three planets at different semi-major axes and eccentricities.
- **Steps:** Many ticks spanning multiple orbital periods.
- **Expected:** Each planet traces its own independent orbit; no cross-coupling of positions.

### 3.6 Multiple moons around one planet
- **Initial state:** One star, one planet, three moons.
- **Steps:** Many ticks.
- **Expected:** Moons remain in distinct orbits relative to the planet; no interference.

### 3.7 Recursive parent resolution
- **Initial state:** Moon `M1` whose `parent = Planet(P1)`. Planet `P1` parent = `Star(S1)`.
- **Steps:** One tick.
- **Expected:** `M1.position_m` = `S1.position_m` + `P1.offset` + `M1.offset`.

### 3.8 Missing parent handling
- **Initial state:** Planet with `parent = Star(missing_id)`.
- **Steps:** One tick.
- **Expected:** Planet stays at `Vector3::ZERO` (or last known); no panic; `OrbitalChanged` event emitted.

### 3.9 Invalid BodyReference handling
- **Initial state:** Planet with `parent = Body(BodyId)` for an unresolved generic body.
- **Steps:** One tick.
- **Expected:** Same graceful degradation as 3.8. Event emitted so diagnostics can alert the user.

### 3.10 Missing orbital period auto-derivation
- **Initial state:** Planet with `orbital_period_s = None`, valid `semi_major_axis_m` and `eccentricity`.
- **Steps:** `initialize()` then `update()`.
- **Expected:** After `update()`, `orbit.orbital_period_s` is `Some(value)` derived from `kepler_period`; position is non-zero.

### 3.11 Planet at periapsis / apoapsis at correct timestamps
- **Initial state:** Elliptical planet; initial time chosen so that `ν = 0` (periapsis) at `t = 0`.
- **Steps:** Advance to `t = T/2` (should be apoapsis).
- **Expected:** Distance from parent at apoapsis equals `a(1+e)` within tolerance.

### 3.12 Velocity direction is tangential
- **Initial state:** Circular planet at periapsis.
- **Steps:** One tick.
- **Expected:** `velocity_m_s` is roughly perpendicular to `(position_m - parent_position)` within 1e-3 rad.

### 3.13 Event publishing only on change
- **Initial state:** Planet with valid orbit.
- **Steps:** Two ticks where orbital positions differ by > 1e-12 m but are stable.
- **Expected:** `OrbitalChanged` events published each tick. If a body is unchanged from a previous tick (zero delta), no duplicate event is required but publishing is acceptable.

### 3.14 Clock pause / resume does not corrupt positions
- **Initial state:** Planet orbiting, clock paused after several ticks.
- **Steps:** `module.update()` with paused context.
- **Expected:** Position unchanged from pre-pause value; no NaN.

---

## 4. Determinism Tests

Location: `worldsmith-stellar/tests/orbital_determinism.rs`

### 4.1 One 10-second fixed-step update vs ten 1-second updates
- **Setup:** Fixed timestep = 1 s for both runs.
  - Run A: advance clock by 10 s; call `module.update()` ten times with `delta = 1`.
  - Run B: call `module.update()` once with `delta = 10` after setting `timestamp_s = 10`.
- **Verification:** Planet and moon positions identical within 1e-9.

### 4.2 Ten 1-second updates vs one-hundred 0.1-second updates
- **Setup:** Same initial state.
  - Run A: 10 × `delta = 1`.
  - Run B: 100 × `delta = 0.1`.
- **Verification:** Final positions identical within 1e-9. Accumulation of round-off must not grow with step count.

### 4.3 Replay consistency
- **Setup:** Record `SimulationSnapshot` sequence for 100 ticks.
- **Steps:** Replay from the same `WorldState` replica using the same seed.
- **Verification:** Each replayed snapshot matches the original snapshot on every field of `PlanetSnapshot` and `Moon`, including `position_m` and `velocity_m_s`, within 1e-12 absolute.

### 4.4 Save/load consistency
- **Setup:** Run simulation for 500 ticks, serialize `SimulationSnapshot` to JSON/Bincode, deserialize, resume 500 more ticks.
- **Steps:** Compare trajectory against an uninterrupted 1000-tick run.
- **Verification:** Position at tick 1000 is identical within 1e-9.

### 4.5 Snapshot consistency — clock vs module timestamp
- **Setup:** Confirm `WorldState::clock.elapsed_seconds()` equals `ModuleContext.timestamp_s` inside the module during update.
- **Verification:** Every tick, recorded `timestamp_s` matches snapshot metadata. No drift.

### 4.6 Cross-platform determinism (simulated)
- **Setup:** Run with forced `f64` precision; compare two independent instances with same seed.
- **Verification:** Bit-identical position sequences (not just within tolerance).
- **Note:** Best effort; host-dependent `libm` can vary. Document acceptable platforms.

---

## 5. Physical Validation Tests

Location: `worldsmith-stellar/tests/orbital_physics.rs`

### 5.1 Circular orbit closure after one period
- **Initial state:** Circular orbit, body at periapsis at `t=0`.
- **Steps:** Advance `T` seconds.
- **Expected:** Position matches initial position within 1 mm.
- **Pass criteria:** `distance(start, end) < 1e-3 m`.

### 5.2 Elliptical orbit periapsis / apoapsis at correct radii
- **Initial state:** Elliptical orbit with `a = 2 AU`, `e = 0.25`.
- **Steps:** Sample radius across 1000 small timesteps.
- **Expected:** Minimum radius = `a(1-e)`; maximum radius = `a(1+e)`.
- **Pass criteria:** Observed min/max within 1 mm.

### 5.3 Velocity highest at periapsis
- **Initial state:** Same elliptical orbit as 5.2.
- **Steps:** Sample `|velocity_m_s|` across 1000 timesteps.
- **Expected:** Maximum magnitude at periapsis.
- **Pass criteria:** Peak magnitude > `sqrt(GM / (a(1-e)))` in closed form; peak occurs within one timestep of periapsis.

### 5.4 Velocity lowest at apoapsis
- **Initial state:** Same orbit.
- **Expected:** Minimum magnitude at apoapsis.
- **Pass criteria:** Valley magnitude < `sqrt(GM / (a(1+e)))` in closed form.

### 5.5 Inclination rotates the orbital plane
- **Initial state:** `inclination = π/4`, `e = 0`, orbit aligned along X at `t=0`.
- **Steps:** Sample positions across one full orbit.
- **Expected:** Maximum orbital Z equals `sma_m * sin(π/4)`; X-Y projection remains circular.
- **Pass criteria:** `max(|z|) - sma_m * sin(π/4) < 1e-6 m`.

### 5.6 Orbit remains centered on its parent body
- **Initial state:** Planet parent at `[1e9, 2e9, 3e9]`, planet orbit `a = 1 AU`.
- **Steps:** Compute barycenter = (parent_mass * parent_pos + planet_mass * planet_pos) / (parent_mass + planet_mass).
- **Expected:** Barycenter moves essentially with parent (planet mass is negligible).
- **Pass criteria:** Parent-centric residual distance `|planet_pos - parent_pos|` matches orbital radius within tolerance; barycenter drift < 1 m.

### 5.7 Energy conservation over many orbits
- **Initial state:** High-eccentricity orbit (`e = 0.8`), `a = 1 AU`, parent mass = solar.
- **Steps:** 10 orbits with small fixed timestep.
- **Expected:** Specific orbital energy `0.5*v² - μ/r` remains constant.
- **Pass criteria:** Standard deviation < 1 J/kg.

### 5.8 Angular momentum conservation (z-component constant for 2D plane)
- **Initial state:** Inclination = 0.
- **Steps:** Position cross velocity should have constant Z.
- **Expected:** Angular momentum about parent stays constant.
- **Pass criteria:** Variation < 1e-9 relative.

### 5.9 Kepler's third law empirically verified
- **Initial state:** Vary `a` from `0.5 AU` to `5 AU`.
- **Steps:** Measure period from time between two consecutive periapsis passages (zero crossings of radial-velocity sign).
- **Expected:** Computed period matches `kepler_period` within 1e-9.
- **Pass criteria:** `|measured - analytical| / analytical < 1e-9`.

---

## 6. Edge Cases

Location: `worldsmith-stellar/tests/orbital_edge_cases.rs`

### 6.1 Zero eccentricity
- **Input:** `e = 0.0`, `a = 1 AU`.
- **Expected:** Position magnitude constant; velocity magnitude constant; `velocity ⊥ radius` always.
- **Fail mode:** Eccentric anomaly solver division by zero if branch not handled.

### 6.2 Near-unity eccentricity
- **Input:** `e = 0.999999`, `a = 1 AU`, `period_s` valid.
- **Expected:** Body spends most time near apoapsis; periapsis is tight but finite. Convergence of eccentric anomaly solver must still happen.
- **Fail mode:** Newton divergence if initial guess is `M` instead of `π` when `e` is high.

### 6.3 Extremely small semi-major axis
- **Input:** `a = 1 m`, `e = 0.0`, parent mass = Earth.
- **Expected:** Orbit scales to 1 m; velocity scales to circular orbit velocity at surface. No underflow of `G * M / a³`.
- **Fail mode:** Underflow to zero in force/acceleration terms if intermediate computation uses `a³`.

### 6.4 Extremely large semi-major axis
- **Input:** `a = 1e18 m`, `e = 0.0`.
- **Expected:** Position stays finite; orbit visually flat over simulation scale. No overflow in `sma³`.
- **Fail mode:** `f64::INFINITY` if intermediate multiplies overflow. Use `hypot` or scaled math.

### 6.5 Very short orbital period
- **Input:** `T = 1e-3 s`, `a` near primary surface.
- **Expected:** After many short-fixed timesteps, closure still holds after 1 period.
- **Fail mode:** Accumulator drift; mean-anomaly aliasing at sub-epsilon timesteps.

### 6.6 Very long orbital period
- **Input:** `T = 1e12 s` (~30,000 years).
- **Expected:** Propagation at `t = 1e12` returns finite velocity; no catastrophic cancellation in `(t / T)` if `t` and `T` are comparable magnitude but non-modular arithmetic handles large numerators.

### 6.7 Zero timestep
- **Input:** `delta_seconds = 0.0` in `ModuleContext`.
- **Expected:** Module still runs; positions unchanged from prior state; no division by zero.
- **Fail mode:** `timestamp_s` unchanged but code branches on `delta_seconds` produce NaN.

### 6.8 Large timestamp values
- **Input:** `timestamp_s = 1e15 s`.
- **Expected:** Mean anomaly reduction still accurate; no precision loss so large that `t mod T` collapses to zero for all `t`.
- **Fail mode:** Floating-point precision degradation for orbital phase.

### 6.9 Floating-point precision limits
- **Input:** Semi-randomized high-eccentricity orbits; compute position from two mathematically equivalent formulas:
  1. Standard Kepler chain (mean → eccentric → true → radius).
  2. Alternative formulation using `atan2` with direct `cosE`/`sinE`.
- **Expected:** Difference between the two results under `1e-9 * sma_m`.
- **Pass criteria:** No catastrophic cancellation; both produce finite values.

---

## 7. Integration Tests

Location: `worldsmith-stellar/tests/orbital_pipeline.rs`

### 7.1 Engine → OrbitalDynamicsModule → WorldState → SimulationSnapshot
- **Setup:** `EngineBuilder` with StellarModule + PlanetFormationModule + OrbitalDynamicsModule.
- **Steps:** Run `tick_fixed()` twice; capture snapshots.
- **Expected:** Both snapshots contain non-zero `Planet.position_m` and `Planet.velocity_m_s`. WorldState contains same values as snapshot clone.

### 7.2 Visualization bridge reads only snapshot positions
- **Setup:** After 7.1, construct `DefaultSnapshotBridge` from the snapshot.
- **Steps:** Inspect bridge output scene.
- **Expected:** `SceneBody.position` for planets/moons equals `snapshot.planets[i].position_m` (converted units) with no independent projection logic in the bridge.
- **Verification:** Fail if bridge code contains any `sin`, `cos`, `atan`, or orbital-element-based position calculation. This is a structural invariant.

### 7.3 No renderer computes orbital projection
- **Setup:** Build `worldsmith-visualization` and scan the compiled binary / source for orbital math usage outside `worldsmith-math` consumers.
- **Steps:** Static analysis or search for `circular_orbit_state`, `elliptical_orbit_state`, `propagate_orbit_state` references in `worldsmith-visualization` and `worldsmith-render` (or their equivalents in this repo).
- **Expected:** Zero such references. The only references are in `worldsmith-math` and `worldsmith-stellar`.

### 7.4 Pipeline stage ordering guarantees orbital module runs last
- **Setup:** Register StellarModule (priority 100), PlanetFormationModule (priority 200), OrbitalDynamicsModule (priority 300), and a synthetic `OrbitalElementChangerModule` (priority 250) that flips `semi_major_axis_m` on a planet.
- **Steps:** Run initialize + one tick.
- **Expected:** Position is computed from the flipped semi-major axis, confirming the orbital module ran after `OrbitalElementChangerModule`.

### 7.5 Moon position relative to planet in snapshot
- **Setup:** One star, one planet, one moon.
- **Steps:** Build snapshot after tick.
- **Expected:** In the snapshot, `moon.position_m - planet.position_m` magnitude matches moon's orbital semi-major axis within tolerance.

### 7.6 Snapshot includes position fields after model change
- **Setup:** Query schema of `SimulationSnapshot` planets.
- **Expected:** `PlanetSnapshot` (via `Planet`) contains `position_m` and `velocity_m_s` after model addition.

---

## 8. Performance Tests

Location: `worldsmith-stellar/benches/orbital_bench.rs`

### 8.1 100 bodies
- **Setup:** 1 star, 33 planets, 66 moons.
- **Metric:** Wall-clock time per `module.update()` call.
- **Target:** < 1 ms per tick on CI-grade hardware.

### 8.2 1,000 bodies
- **Setup:** 1 star, 200 planets (average 4 moons each).
- **Metric:** Wall-clock time per tick.
- **Target:** < 5 ms per tick.

### 8.3 10,000 bodies
- **Setup:** 1 star, 1,000 planets (average 9 moons each).
- **Metric:** Wall-clock time per tick.
- **Target:** < 50 ms per tick.

### 8.4 100,000 bodies
- **Setup:** 10 stars, 20,000 planets (average 4 moons each).
- **Metric:** Wall-clock time per tick.
- **Target:** < 500 ms per tick. If above, investigate O(B²) parent-lookup or allocation hot spots.

### 8.5 Allocation behavior
- **Setup:** Profile 10,000-body run with `valgrind --tool=massif` or OS-equivalent heap profiler.
- **Expected:** No per-tick heap allocation in the hot path. Module-local buffers (parent lookup maps) should be reused across ticks.
- **Pass criteria:** Heap growth across 100 ticks < 1 KB per tick (excluding snapshot cloning, which is outside the module).

### 8.6 Potential optimization watchpoints (not implemented)
- **Parent lookup table:** If bodies use `BTreeMap`, iteration is already O(B log B). Consider temporary `HashMap` built once per tick if profiling shows `BTreeMap::get` hot.
- **Event deduplication:** Publishing `OrbitalChanged` every tick for every body is O(B). Downstream modules should query state, not maintain duplicate maps.
- **Pooled `Vector3` / orbit-state allocations:** If vectors are boxed, pools may help. Expected `Vector3` is stack-inline; expect zero allocation here.
- **SIMD:** Single-body math is scalar; SIMD only helps if many bodies are resolved in parallel. Not a Phase 9 target.

---

## 9. Regression Tests

Location: `worldsmith-stellar/tests/orbital_regression.rs`

### 9.1 Deterministic output invariant
- **Setup:** Thousands of ticks across representative preset.
- **Expected:** Bit-identical trajectory across engine commits unless orbital implementation changes intentionally.

### 9.2 Stable parent hierarchy
- **Setup:** Parent-child chain with depth 3 (System → Star → Planet → Moon).
- **Expected:** Resolution stops at first `Star` or `System` and does not loop or panic.

### 9.3 Correct snapshot propagation
- **Setup:** Run + snapshot.
- **Expected:** `snapshot.planets[*].position_m` equals `WorldState.planets[*].position_m` within serialization tolerance.

### 9.4 No NaN coordinates
- **Setup:** Many-body system with mixed valid / partially valid orbital elements.
- **Expected:** No `NaN` or `Infinity` in any `position_m` or `velocity_m_s` field after any tick.

### 9.5 No infinite velocities
- **Setup:** Orbit with `e = 0.999999`, `a = 1 AU`.
- **Expected:** Velocity finite everywhere; periapsis velocity obeys vis-viva upper bound.

### 9.6 No duplicated orbital computation outside simulation
- **Setup:** Static + runtime audit.
- **Expected:**
  - `worldsmith-math::orbital` is the **only** place implementing orbit propagation helpers.
  - `worldsmith-visualization` and presentation crates contain no orbital math.
  - `worldsmith-models` contains no orbital math.

### 9.7 Module initialization caches `orbital_period_s`
- **Setup:** Planet without pre-set `orbital_period_s`. Run `initialize()` then `update()`.
- **Expected:** After initialize, `planet.orbit.orbital_period_s` is populated. Subsequent ticks do not change it.

---

## 10. Future Test Coverage

| Feature | Test focus |
|---------|------------|
| **Binary stars** | Mutual orbit with masses A and B; both positions offset from barycenter; barycenter fixed or moving; semi-major axes sum to separation. |
| **N-body gravity** | Three-body initial conditions (e.g., figure-8); trajectory remains stable and matches numerical integrator golden. |
| **Lagrange points** | Derived positions of L4/L5 relative to primary-secondary line; stability over time. |
| **Tidal locking** | Spin period evolves to orbital period; energy dissipation monotonically lowers semi-major axis / eccentricity over long runs. |
| **Planet migration** | `semi_major_axis_m` changes incrementally; positions remain continuous and bounded. |
| **Relativistic corrections** | Perihelion precession of Mercury-like orbit matches GR prediction within 1 arcsecond/century. |
| **Procedural galaxy generation** | Millions of stars in a galactic potential; orbital propagation maintains spiral-arm morphology; performance does not regress. |
| **Save/load replay delta** | Every save produces a replay that matches the original trajectory exactly. |
| **Mass change event** | EventPayload that modifies star mass; planet immediately orbits new focus within 1 tick. |
| **Collision / accretion event** | Parent body removed; child either transitions to new parent or is marked invalid without NaN. |

---

## 11. Prioritized Implementation Checklist

Implement these tests **before** merging the OrbitalDynamicsModule.

**P0 — Must pass before merge**
- [ ] Unit: `mean_anomaly_from_time` at basic, negative, and large timestamps
- [ ] Unit: `eccentric_anomaly_from_mean` circular, elliptical, high-eccentricity
- [ ] Unit: `true_anomaly_from_eccentric` circular and elliptical
- [ ] Unit: `propagate_orbit_state` circular and elliptical agreement with existing helpers
- [ ] Integration: Planet position non-zero after one tick
- [ ] Integration: Moon follows moving parent
- [ ] Integration: Multiple planets around same star do not cross-couple
- [ ] Integration: Missing parent handling (no panic, graceful fallback)
- [ ] Determinism: One 10-second update == ten 1-second updates
- [ ] Determinism: Replay consistency over 100+ ticks
- [ ] Physics: Circular orbit closure after one period
- [ ] Physics: Elliptical periapsis/apoapsis radii correct
- [ ] Physics: Velocity highest at periapsis, lowest at apoapsis
- [ ] Regression: No NaN / no infinite velocities
- [ ] Regression: No orbital math in visualization crate

**P1 — Must pass before Phase 9.5 review**
- [ ] Unit: Inclination rotation correctness
- [ ] Unit: Velocity magnitude energy conservation
- [ ] Integration: Recursive parent resolution 3-level deep
- [ ] Integration: Pipeline ordering — orbital module runs last
- [ ] Integration: Visualization bridge reads snapshot positions only
- [ ] Determinism: Save/load consistency 1000 ticks
- [ ] Determinism: Cross-platform tolerance documented
- [ ] Physics: Inclination rotates orbital plane correctly
- [ ] Physics: Orbit centered on parent body
- [ ] Physics: Kepler's third law empirically verified
- [ ] Edge: Zero and near-unity eccentricity
- [ ] Edge: Very short and very long orbital periods
- [ ] Regression: Snapshot propagation invariant
- [ ] Regression: period auto-derivation during initialize

**P2 — Must pass before Phase 10**
- [ ] Unit: Numerical stability — extreme semi-major axis
- [ ] Unit: Floating-point precision limits
- [ ] Integration: Engine → WorldState → SimulationSnapshot full pipeline
- [ ] Integration: Moon position relative to planet in snapshot
- [ ] Determinism: Clock pause/resume does not corrupt positions
- [ ] Physics: Energy conservation over 10 orbits
- [ ] Physics: Angular momentum conservation
- [ ] Edge: Missing `orbital_period_s` auto-derivation
- [ ] Performance: 100 bodies < 1 ms baseline
- [ ] Performance: 1,000 / 10,000 / 100,000 body scaling benchmarks
- [ ] Regression: No duplicated orbital computation outside simulation
- [ ] Regression: Stable parent hierarchy depth 3+

**P3 — Future / deferred**
- [ ] Binary star mutual orbits
- [ ] N-body gravity golden tests
- [ ] Lagrange point placement
- [ ] Tidal locking long-run behavior
- [ ] Planet migration smoothness
- [ ] Relativistic perihelion precession
- [ ] Galaxy-scale procedural stress test
- [ ] Mass-change event continuity
- [ ] Collision/accretion transition behavior
