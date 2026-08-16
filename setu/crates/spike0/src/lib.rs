//! Spike 0: RLNC Overhead and QUIC Migration Benchmarks

pub mod rlnc_bench;
pub mod latency_bench;
pub mod migration_bench;

pub fn run_all_benchmarks() {
    println!("Running Spike 0 Benchmark Suite...");
    rlnc_bench::run();
    latency_bench::run();
    migration_bench::run();
}
