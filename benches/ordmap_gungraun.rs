//! Instruction counts and simulated cache misses of `OrdMap` operations, measured with
//! Gungraun (Callgrind). See `gungraun_ordmap/mod.rs` for the operations.
//!
//! Needs Valgrind and `cargo install gungraun-runner` at the version of the `gungraun`
//! dev-dependency. Run with `cargo bench --bench ordmap_gungraun`; pass
//! `-- --save-baseline=NAME` and later `-- --baseline=NAME` to compare two builds.
use gungraun::main;
use imbl::OrdMap;

mod utils;
use utils::*;

#[macro_use]
mod gungraun_ordmap;
use gungraun_ordmap::Str;

ordmap_suite!(i64_benches, ordmap_i64, OrdMap, i64);
ordmap_suite!(str_benches, ordmap_str, OrdMap, Str);

main!(
    config = gungraun_ordmap::config(),
    library_benchmark_groups = [ordmap_i64, ordmap_str]
);
