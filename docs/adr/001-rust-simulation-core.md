# Architecture Decision Record 001: Rust Simulation Core

## Status

Accepted — Phase 1

## Context

WorldSmith requires deterministic, high-performance simulation of coupled physical systems over long timescales. The core must be suitable for open-source scientific use, headless batch runs, and eventual FFI bindings.

## Decision

Implement all simulation logic in **Rust** as a Cargo workspace of focused crates. Presentation (wgpu, egui) and I/O are separate crates that depend on data models and snapshots, not on solvers.

## Consequences

**Positive:**
- Deterministic execution without GC pauses
- Compiler-enforced module boundaries
- Single language for core + CLI; optional Python FFI later
- Strong ecosystem for serialization and parallel grid ops

**Negative:**
- Steeper learning curve for contributors unfamiliar with Rust
- Chemistry and legacy scientific libraries may require Rust ports or careful FFI

## Alternatives Considered

| Option | Rejected because |
|--------|------------------|
| C++ | Faster to find libs, but weaker boundary enforcement and memory safety burden |
| Python core | GC and float semantics complicate determinism at scale |
| Bevy all-in-one | Couples ECS/game loop to scientific pipeline; wrong abstraction |
