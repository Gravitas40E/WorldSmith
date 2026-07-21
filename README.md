# WorldSmith

A deterministic, physics-first procedural planet simulation engine.

WorldSmith generates planetary systems where every property—orbit, composition, surface, atmosphere, and climate—emerges from simulated physical processes, not procedural noise.

## Status

**Phase 1 — Architecture.** The project structure and design documents are in place. Simulation implementation has not begun.

See [ARCHITECTURE.md](./ARCHITECTURE.md) for the complete system design.

## Project Structure

```
crates/          Rust workspace members (simulation, render, ui, io, …)
assets/          Shaders, presets, chemistry data (no code)
docs/            Architecture decision records and design deep-dives
tests/           Cross-crate integration tests
tools/           Schema validators and dev utilities
```

## License

To be determined — open-source intent.
