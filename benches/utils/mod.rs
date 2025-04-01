#![allow(dead_code)]

pub mod handwritten;

pub use tiny_bench::black_box;

/// Runs a benchmark without extremely slow and unnecessary warm-up.
pub fn bench<T, R, F, S>(label: &'static str, setup: S, closure: F, compare: bool)
where
    F: FnMut(R) -> T,
    S: FnMut() -> R,
{
    let cfg = tiny_bench::BenchmarkConfig {
        dump_results_to_disk: compare,
        warm_up_time: core::time::Duration::ZERO,
        ..Default::default()
    };

    tiny_bench::bench_with_setup_configuration_labeled(label, &cfg, setup, closure);
}

/// Runs a benchmark that compares two or more runs with the same label.
/// To do this, tiny-bench just records the last run to disk and then reads it
/// back, so multiple runs without clearing target data will generate a delta on
/// each subsequent run.
pub fn compare<T, R, F, S>(label: &'static str, setup: S, closure: F)
where
    F: FnMut(R) -> T,
    S: FnMut() -> R,
{
    bench(label, setup, closure, true);
}

/// Runs a one-shot benchmark.
pub fn one_shot<T, R, F, S>(label: &'static str, setup: S, closure: F)
where
    F: FnMut(R) -> T,
    S: FnMut() -> R,
{
    bench(label, setup, closure, false);
}

/// Repeats the given closure several times.
///
/// We do this in order to measure benchmarks that require at least some
/// amount of nanoseconds to run through.
#[inline(always)]
pub fn repeat<F>(mut f: F)
where
    F: FnMut(),
{
    for _ in 0..10 {
        f();
    }
}
