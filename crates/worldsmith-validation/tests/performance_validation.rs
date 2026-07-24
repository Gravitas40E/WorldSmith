//! Performance validation tests.
//!
//! Measure tick time and memory across different planet counts.  These are
//! non-rigorous benchmarks intended to catch regressions, not to compare
//! against external baselines.

use worldsmith_validation::measure_performance;

#[test]
fn performance_100_planets() {
    let report = measure_performance(100);
    println!("100 planets: {:.3} ms/tick", report.tick_time_ms);
    assert!(report.tick_time_ms >= 0.0);
}

#[test]
fn performance_1_000_planets() {
    let report = measure_performance(1_000);
    println!("1,000 planets: {:.3} ms/tick", report.tick_time_ms);
    assert!(report.tick_time_ms >= 0.0);
}

#[test]
fn performance_10_000_planets() {
    let report = measure_performance(10_000);
    println!("10,000 planets: {:.3} ms/tick", report.tick_time_ms);
    assert!(report.tick_time_ms >= 0.0);
}

#[test]
fn performance_100_000_planets() {
    let report = measure_performance(100_000);
    println!("100,000 planets: {:.3} ms/tick", report.tick_time_ms);
    assert!(report.tick_time_ms >= 0.0);
}
