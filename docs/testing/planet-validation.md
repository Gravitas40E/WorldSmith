# Planet Validation Framework

## Purpose

The `worldsmith-validation` crate provides a deterministic scientific validation
framework for the WorldSmith Planet Evolution subsystem (Phases 10C–10G).

Validation does not change runtime behavior.  It observes, measures, and asserts.

## Validation Categories

### State Validation

Temperatures, fluxes, and rates must remain finite.  No NaN.  No Inf.  Option
state must be structurally consistent.

### Ownership Validation

Every `FieldKey` must have exactly one declared runtime writer.  Duplicate
ownership violates ADR-011.

### Scientific Invariants

Inequalities that must hold independent of numerical realism:

- `core_temperature >= mantle_temperature`
- `volcanic_flux >= 0`
- `plate_velocity >= 0`
- `crustal_recycling_rate >= 0`
- `radiogenic_heat >= 0`
- `internal_heat >= 0`

### Long-term Stability

Simulate 100 / 1,000 / 10,000 / 100,000 ticks and verify:

- no divergence,
- no NaN / Inf,
- no oscillation,
- finite values.

### Deterministic Replay / Golden Simulation

Identical seed, state, and timestep sequence must produce bit-for-bit identical
outputs.

Golden worlds used:

- Earth-like (1 Earth mass, 1 AU, rocky)
- Mars-like (0.107 Earth mass, 1.52 AU, rocky)
- Super Earth (5 Earth masses, 0.8 AU, rocky)

### Cross-module Validation

Verify that:

- no module writes another module's owned fields,
- the dependency graph Core -> Mantle -> Volcanism -> Plate Tectonics is
  enforced at the declared interface level.

### Performance Validation

Measure tick time / memory / allocation counts at:

- 100 planets
- 1,000 planets
- 10,000 planets
- 100,000 planets

These are regression-sensitive measurements.  They are not expected to match
external baselines.

## Philosophy

### Validation != Scientific Realism

Validation verifies structural correctness: determinism, consistency, and basic
inequalities.  Whether the equations match reality is a separate scientific
review task.

### Determinism Philosophy

Given identical:

- seed / deterministic builder configuration,
- initial state,
- timestep sequence,

outputs must be bit-for-bit identical across separate engine runs.

## Integration

- No scheduler changes.
- No planetary physics changes.
- Pure test / helper code.
- Existing evolution modules remain unchanged in runtime behavior.
