//! Deterministic replay / golden simulation tests.

use worldsmith_validation::{deterministic_replay, GoldenWorld, ReplayOutcome};

#[test]
fn deterministic_replay_matches_after_100_ticks() {
    let golden = GoldenWorld::earth_mars_super_earth();
    let (outcome, _, _) = deterministic_replay(&golden, 100);
    assert_eq!(outcome, ReplayOutcome::Matches);
}

#[test]
fn deterministic_replay_matches_after_1_000_ticks() {
    let golden = GoldenWorld::earth_mars_super_earth();
    let (outcome, _, _) = deterministic_replay(&golden, 1_000);
    assert_eq!(outcome, ReplayOutcome::Matches);
}

#[test]
fn deterministic_replay_matches_after_10_000_ticks() {
    let golden = GoldenWorld::earth_mars_super_earth();
    let (outcome, _, _) = deterministic_replay(&golden, 10_000);
    assert_eq!(outcome, ReplayOutcome::Matches);
}

#[test]
fn deterministic_replay_matches_after_100_000_ticks() {
    let golden = GoldenWorld::earth_mars_super_earth();
    let (outcome, _, _) = deterministic_replay(&golden, 100_000);
    assert_eq!(outcome, ReplayOutcome::Matches);
}
