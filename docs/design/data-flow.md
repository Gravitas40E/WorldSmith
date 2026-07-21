# Data Flow

This document expands the data-flow section of [ARCHITECTURE.md](../../ARCHITECTURE.md).

## Three Channels

WorldSmith uses three distinct communication channels. Mixing them causes hidden coupling and breaks determinism.

| Channel | Direction | Mutability | Used by |
|---------|-----------|------------|---------|
| **State fields** | Module ↔ `WorldState` | Mutable (during tick) | All domain modules |
| **Events** | Module → Engine → Module | Append-only queue | Cross-domain side effects |
| **Snapshots** | Engine → Render/UI | Immutable | Presentation only |

## State Field Lifecycle

```
1. Pipeline stage or tick begins
2. Engine validates module's declared reads/writes against registry
3. Module reads input fields (shared reference)
4. Module writes output fields (exclusive reference to owned buffers)
5. Engine marks fields dirty for downstream scheduling
6. Events flushed before next tick boundary
```

## Snapshot Build

Snapshots are built **after** tick completion, never during solver iteration:

```
WorldState (complete tick)
    → visualization::build_snapshot(state, layer_mask)
    → VisualSnapshot { meshes, textures, scalar_ranges, labels }
    → send to render thread (double-buffer swap)
```

## Command Flow

```
UI/CLI
    → EngineCommand queue
    → engine::process_commands() at tick boundary
    → validated mutation OR pipeline re-schedule
```

Commands that change initial conditions trigger a **controlled pipeline restart** from the affected stage forward—not a full random regeneration.
