# Planet Evolution Test Plan

## Scope
Covers every module that will be introduced in Phase 10: core, mantle, tectonics,
volcanism, atmosphere, hydrology, climate, magnetic field, and biosphere.

The framework must guarantee **determinism**, **scientific sanity**, and
**deterministic replay equality**.

---

## 1. Unit Tests
Per-module, in-module `#[cfg(test)]`.

| Test | What it verifies |
|------|------------------|
| `initialize_populates_local_state` | Module local state derives from initial `WorldState` without mutation side-effects. |
| `update_advances_time` | After one tick with known delta, tracked properties change in the expected direction. |
| `update_publishes_events` | Significant changes emit the correct `SimulationEvent` kinds with correct targets. |
| `update_ignores_unrelated_events` | Foreign events are not consumed. |
| `invalid_input_returns_error` | Core temperature = 0 K produces a typed error, never a NaN write. |
| `determinism_fixed_seed` | Run 100 ticks twice with identical seed and state; outputs match bit-for-bit. |

---

## 2. Regression Tests
Global `tests/` directory in `worldsmith-evolution`.

| Test | Purpose |
|------|---------|
| `evolution_after_formation` | `PlanetFormationModule` → all evolution ticks → snapshot fields are initialized, not left at defaults. |
| `evolution_preserves_orbit` | Positions from `OrbitalDynamicsModule` are unchanged by any evolution module. |
| `event_chain_atmosphere_after_volcanism` | `VolcanismModule` publishes event → `AtmosphereModule` consumes and updates pressure. |
| `snapshot_reflects_worldstate` | After evolution ticks, `WorldState::snapshot()` contains non-default `Planet` values. |
| `no_double_write` | No two running evolution modules write the same `Planet` field in the same tick (scheduler validation). |

---

## 3. Scientific Validation Tests
Not asserting exact numbers (which would overfit), but asserting **physical constraints**.

| Test | Constraint |
|------|------------|
| `core_cooling_monotonic` | Core temperature never increases unless active heating is modeled. |
| `volcanic_outgassing_conserves_mass` | Atmosphere addition mass does not exceed outgassed inventory. |
| `greenhouse_within_limits` | Computed temperature difference follows Stefan-Boltzmann bounds. |
| `hydrology_water_conservation` | Total water mass is constant unless escape or delivery events apply. |
| `tectonic_activity_bounded` | Tectonic slug ≤ 1.0, never NaN. |

---

## 4. Determinism Tests
| Test | Protocol |
|------|----------|
| `replay_identical_output` | Run formation + 1000 Myr of evolution; hash the entire `SimulationSnapshot` vector. Repeat three times; hashes must match on the same machine. |
| `multi_rate_ordering_independent` | Shuffle module wake ordering (within valid priorities) and ensure final snapshot matches sequential ordering. |
| `event_ordering_deterministic` | Emit two events with the same timestamp but different ids; verify consumption order respects id ordering. |

---

## 5. Performance Benchmarks
Using `criterion` in each module crate.

| Benchmark | Baseline target |
|-----------|-----------------|
| `core_mantle_100_planets` | < 5 ms |
| `core_mantle_1000_planets` | < 30 ms |
| `climate_10000_planets` | < 200 ms |
| `full_pipeline_1000_planets_1k_ticks` | < 30 s total |

Track regression with `criterion` baselines in CI.

---

## 6. Integration Tests
| Test | Purpose |
|------|---------|
| `full_evolution_pipeline` | Formation → Orbital → all evolution ticks → valid snapshot. |
| `parallel_planet_iteration` | 1000 planets with parallel iterator; no data races, identical output to seq. |
| `save_reload_evolution_continues` | Save after 500 Myr, reload, tick another 500 Myr; end state equals a single 1000 Myr run. |

---

## 7. Golden / Snapshot Tests
Fixed-seed runs produce a saved `SimulationSnapshot` JSON.  CI compares key fields to tolerance 1e-9 relative.

## 8. Static & Safety Checks
- `cargo clippy` must pass.
- `cargo fmt` must pass.
- `cargo test --workspace` must pass.
- `cargo check --workspace` must pass.

---

## Success Criteria
1. All new modules pass unit tests.
2. Determinism tests pass on three independent runs.
3. Benchmark targets are met for 1k planets.
4. No existing `worldsmith-planet` tests break.
5. A developer can add a `BiosphereModule` in `worldsmith-biosphere` without opening a single file in `worldsmith-evolution` or `worldsmith-planet`.
