//! The `OrdMap` operations shared by the Gungraun benches.
//!
//! Callgrind runs each benchmark once and counts what it executes, so the results don't
//! depend on machine noise. Every benchmark builds its input in a setup function, which
//! Gungraun leaves out of the measurement, and returns its input so that dropping it isn't
//! measured either. The setups also run the measured operation once, which warms up the
//! simulated caches and the allocator to match the steady state Criterion measures.
//!
//! The operations and inputs mirror `benches/ordmap.rs`, with the same names where both
//! suites have a benchmark: keys come from `utils::TestData`, maps are built from keys in
//! ascending order and lookups, removals and updates visit the keys in `utils::reorder`
//! order. As there, `lookup_ne` looks up keys greater than every key in the map, so it only
//! walks the rightmost path; `lookup_miss` looks up keys that fall between the map's keys.

use std::sync::Arc;

pub type Str = Arc<String>;

/// Defines module `$module` with a Gungraun benchmark group `$group`, which runs the suite on
/// `$map<$key, $key>`, and imports the group for `main!`.
macro_rules! ordmap_suite {
    ($module:ident, $group:ident, $map:ident, $key:ident) => {
        mod $module {
            use super::*;
            use gungraun::{library_benchmark, library_benchmark_group};
            use std::hint::black_box;

            type K = $key;
            type Map = $map<K, K>;

            fn build_map(keys: Vec<K>) -> Map {
                let values = K::generate(keys.len());
                keys.into_iter().zip(values).collect()
            }

            fn get_all(map: &Map, keys: &[K]) {
                for k in keys {
                    black_box(map.get(k));
                }
            }

            /// A map of `n` keys and all its keys, shuffled.
            fn setup_lookup(n: usize) -> (Map, Vec<K>) {
                let keys = K::generate(n);
                let order = reorder(&keys);
                let map = build_map(keys);
                get_all(&map, &order);
                (map, order)
            }

            /// A map of `n` keys and `n` keys greater than all of them, shuffled.
            fn setup_lookup_ne(n: usize) -> (Map, Vec<K>) {
                let mut keys = K::generate(n * 2);
                let order = reorder(&keys.split_off(n));
                let map = build_map(keys);
                get_all(&map, &order);
                (map, order)
            }

            /// A map of `n` keys and `n` keys that fall between them, shuffled.
            fn setup_lookup_miss(n: usize) -> (Map, Vec<K>) {
                let (hits, misses): (Vec<K>, Vec<K>) = K::generate(n * 2)
                    .chunks(2)
                    .map(|pair| (pair[0].clone(), pair[1].clone()))
                    .unzip();
                let order = reorder(&misses);
                let map = build_map(hits);
                get_all(&map, &order);
                (map, order)
            }

            #[library_benchmark(setup = setup_lookup)]
            #[bench::n1000(1000)]
            #[bench::n10000(10000)]
            #[bench::n100000(100000)]
            fn lookup(input: (Map, Vec<K>)) -> (Map, Vec<K>) {
                get_all(&input.0, &input.1);
                input
            }

            #[library_benchmark(setup = setup_lookup_ne)]
            #[bench::n10000(10000)]
            #[bench::n100000(100000)]
            fn lookup_ne(input: (Map, Vec<K>)) -> (Map, Vec<K>) {
                get_all(&input.0, &input.1);
                input
            }

            #[library_benchmark(setup = setup_lookup_miss)]
            #[bench::n1000(1000)]
            #[bench::n10000(10000)]
            fn lookup_miss(input: (Map, Vec<K>)) -> (Map, Vec<K>) {
                get_all(&input.0, &input.1);
                input
            }

            fn insert_all_mut(entries: Vec<(K, K)>) -> Map {
                let mut map = Map::new();
                for (k, v) in entries {
                    map.insert(k, v);
                }
                map
            }

            fn insert_all(entries: Vec<(K, K)>) -> Map {
                let mut map = Map::new();
                for (k, v) in entries {
                    map = map.update(k, v);
                }
                map
            }

            /// `n` entries in ascending key order.
            fn setup_entries(n: usize) -> Vec<(K, K)> {
                K::generate(n).into_iter().zip(K::generate(n)).collect()
            }

            fn setup_insert_mut(n: usize) -> Vec<(K, K)> {
                let entries = setup_entries(n);
                drop(insert_all_mut(entries.clone()));
                entries
            }

            fn setup_insert(n: usize) -> Vec<(K, K)> {
                let entries = setup_entries(n);
                drop(insert_all(entries.clone()));
                entries
            }

            // Inserts `n` keys in ascending order into an empty map.
            #[library_benchmark(setup = setup_insert_mut)]
            #[bench::n1000(1000)]
            #[bench::n10000(10000)]
            fn insert_mut(entries: Vec<(K, K)>) -> Map {
                insert_all_mut(entries)
            }

            // As `insert_mut`, but keeps every intermediate map persistent: `map = map.update(..)`.
            #[library_benchmark(setup = setup_insert)]
            #[bench::n1000(1000)]
            #[bench::n10000(10000)]
            fn insert(entries: Vec<(K, K)>) -> Map {
                insert_all(entries)
            }

            fn remove_all_mut(map: &mut Map, keys: &[K]) {
                for k in keys {
                    map.remove(k);
                }
            }

            fn remove_all(map: &mut Map, keys: &[K]) {
                for k in keys {
                    *map = map.without(k);
                }
            }

            /// A clone of a map of `n` keys, the map itself, and its keys, shuffled. Removing
            /// from the clone copies each node the first time it is touched, as in `ordmap.rs`.
            fn setup_remove(n: usize) -> (Map, Map, Vec<K>) {
                let keys = K::generate(n);
                let order = reorder(&keys);
                let map = build_map(keys);
                (map.clone(), map, order)
            }

            fn setup_remove_mut(n: usize) -> (Map, Map, Vec<K>) {
                let input = setup_remove(n);
                remove_all_mut(&mut input.1.clone(), &input.2);
                input
            }

            fn setup_remove_persistent(n: usize) -> (Map, Map, Vec<K>) {
                let input = setup_remove(n);
                remove_all(&mut input.1.clone(), &input.2);
                input
            }

            #[library_benchmark(setup = setup_remove_mut)]
            #[bench::n1000(1000)]
            #[bench::n10000(10000)]
            fn remove_mut(mut input: (Map, Map, Vec<K>)) -> (Map, Map, Vec<K>) {
                remove_all_mut(&mut input.0, &input.2);
                input
            }

            // As `remove_mut`, but with `map = map.without(..)`.
            #[library_benchmark(setup = setup_remove_persistent)]
            #[bench::n1000(1000)]
            #[bench::n10000(10000)]
            fn remove(mut input: (Map, Map, Vec<K>)) -> (Map, Map, Vec<K>) {
                remove_all(&mut input.0, &input.2);
                input
            }

            /// How many persistent updates `insert_once` and `remove_once` make.
            const ONCE: usize = 100;

            fn update_each(map: &Map, entries: Vec<(K, K)>) {
                for (k, v) in entries {
                    black_box(map.update(k, v));
                }
            }

            fn without_each(map: &Map, keys: &[K]) {
                for k in keys {
                    black_box(map.without(k));
                }
            }

            /// A map of `n` keys and `ONCE` of its keys with new values, shuffled.
            fn setup_insert_once(n: usize) -> (Map, Vec<(K, K)>) {
                let keys = K::generate(n);
                let values = K::generate(n);
                let entries: Vec<_> = reorder(&keys)
                    .into_iter()
                    .zip(reorder(&values))
                    .take(ONCE)
                    .collect();
                let map = keys.into_iter().zip(values).collect();
                update_each(&map, entries.clone());
                (map, entries)
            }

            /// A map of `n` keys and `ONCE` of its keys, shuffled.
            fn setup_remove_once(n: usize) -> (Map, Vec<K>) {
                let keys = K::generate(n);
                let mut order = reorder(&keys);
                order.truncate(ONCE);
                let map = build_map(keys);
                without_each(&map, &order);
                (map, order)
            }

            // `ONCE` times: `map.update(k, v)` on the same map, dropping the result.
            #[library_benchmark(setup = setup_insert_once)]
            #[bench::n1000(1000)]
            #[bench::n10000(10000)]
            #[bench::n100000(100000)]
            fn insert_once(input: (Map, Vec<(K, K)>)) -> Map {
                update_each(&input.0, input.1);
                input.0
            }

            // `ONCE` times: `map.without(k)` on the same map, dropping the result.
            #[library_benchmark(setup = setup_remove_once)]
            #[bench::n1000(1000)]
            #[bench::n10000(10000)]
            #[bench::n100000(100000)]
            fn remove_once(input: (Map, Vec<K>)) -> (Map, Vec<K>) {
                without_each(&input.0, &input.1);
                input
            }

            fn iter_all(map: &Map) {
                for entry in map {
                    black_box(entry);
                }
            }

            fn setup_iter(n: usize) -> Map {
                let map = build_map(K::generate(n));
                iter_all(&map);
                map
            }

            #[library_benchmark(setup = setup_iter)]
            #[bench::n1000(1000)]
            #[bench::n10000(10000)]
            fn iter(map: Map) -> Map {
                iter_all(&map);
                map
            }

            /// `n` entries in shuffled key order.
            fn setup_collect(n: usize) -> Vec<(K, K)> {
                let keys = K::generate(n);
                let values = K::generate(n);
                let entries: Vec<_> = reorder(&keys).into_iter().zip(values).collect();
                drop(entries.clone().into_iter().collect::<Map>());
                entries
            }

            // Builds a map from `n` entries in shuffled key order with `collect()`.
            #[library_benchmark(setup = setup_collect)]
            #[bench::n1000(1000)]
            #[bench::n10000(10000)]
            fn collect(entries: Vec<(K, K)>) -> Map {
                entries.into_iter().collect()
            }

            library_benchmark_group!(
                name = $group,
                benchmarks = [
                    lookup,
                    lookup_ne,
                    lookup_miss,
                    insert_mut,
                    insert,
                    remove_mut,
                    remove,
                    insert_once,
                    remove_once,
                    iter,
                    collect
                ]
            );
        }
        use $module::$group;
    };
}

/// The Callgrind configuration of both Gungraun benches: estimated cycles from the default
/// cache simulation, plus the cache misses behind them and the branch simulation. The count
/// of conditional branches (`Bc`) is exact, but Callgrind's simple branch predictor makes the
/// mispredictions (`Bcm`) shift with code placement, much like wall-clock times.
pub fn config() -> gungraun::LibraryBenchmarkConfig {
    use gungraun::{Callgrind, CallgrindMetrics, LibraryBenchmarkConfig};
    LibraryBenchmarkConfig::default()
        .tool(
            Callgrind::with_args(["--branch-sim=yes"])
                .format([CallgrindMetrics::Default, CallgrindMetrics::CacheMisses]),
        )
        .clone()
}
