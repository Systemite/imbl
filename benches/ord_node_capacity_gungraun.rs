//! `ordmap_gungraun` at node size 64, to compare with the default of 16.
//!
//! Run `cargo bench --bench ordmap_gungraun --bench ord_node_capacity_gungraun` and
//! compare `ordmap_*` with `node_64_*`.
use gungraun::main;
use imbl::{ordmap::GenericOrdMap, shared_ptr::DefaultSharedPtr};

mod utils;
use utils::*;

#[macro_use]
mod gungraun_ordmap;
use gungraun_ordmap::Str;

type OrdMap64<K, V> = GenericOrdMap<K, V, DefaultSharedPtr, 64>;

ordmap_suite!(i64_benches, node_64_i64, OrdMap64, i64);
ordmap_suite!(str_benches, node_64_str, OrdMap64, Str);

main!(
    config = gungraun_ordmap::config(),
    library_benchmark_groups = [node_64_i64, node_64_str]
);
