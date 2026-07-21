# Pipeline Stages

Generation-time pipeline stages for WorldSmith. Runtime ticks use the same `PipelineStage` machinery where appropriate but are driven by the multi-rate scheduler instead of strict ordering.

## Stage Registry

| Stage ID | Crate | Depends on | Primary outputs |
|----------|-------|------------|-----------------|
| `preset_load` | engine | — | Validated initial conditions |
| `stellar_formation` | stellar | preset_load | L, T_eff, spectrum, age |
| `planetary_accretion` | planet | stellar_formation | M, R, orbit, composition, rotation |
| `geology_bulk` | geology | planetary_accretion | Heat flow, crust thickness, tectonic regime |
| `atmosphere_initial` | atmosphere | geology_bulk, chemistry | Composition, P_surface, scale height |
| `climate_initial` | climate | atmosphere_initial, stellar | T_surface field, circulation, ice line |
| `geology_surface` | geology | climate_initial | Erosion, sediment, refined topography |
| `snapshot_emit` | visualization | geology_surface | VisualSnapshot |

## Re-run Policy

When a user edits a parameter via UI:

1. Engine identifies the **earliest affected stage** (e.g., stellar mass change → `stellar_formation`).
2. All downstream stages re-execute in order.
3. Upstream stage outputs are preserved unless invalidated.
4. Determinism: same seed + same parameter set → same outputs (verified by golden tests).

## Stage Context

Each stage receives:

- `&mut WorldState` — write access to owned fields
- `StageContext` — read access to dependency outputs, RNG stream, physical constants
- `StageConfig` — resolution, time limits, convergence tolerances

Stages must not spawn threads that write shared state without engine coordination.

## Convergence

Iterative stages (atmosphere hydrostatic equilibrium, climate radiative balance) declare:

- Maximum iterations
- Convergence metric and tolerance
- Behavior on failure: return `StageError::NonConvergence` — never silently accept partial results
