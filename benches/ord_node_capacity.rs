//! Compare B-tree child storage at the default and a larger node size.
//! Run with `--save-baseline before`, then `--baseline before` after a change.
use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use imbl::{ordmap::GenericOrdMap, shared_ptr::DefaultSharedPtr};
use rand::{rngs::SmallRng, seq::SliceRandom, SeedableRng};
use std::{hint::black_box, time::Duration};

type Map<const N: usize> = GenericOrdMap<u64, u64, DefaultSharedPtr, N>;

fn node_size<const N: usize>(c: &mut Criterion) {
    const LEN: u64 = 32_768;
    let mut keys: Vec<_> = (0..LEN).collect();
    keys.shuffle(&mut SmallRng::seed_from_u64(0x0062_7472_6565));
    let base: Map<N> = keys.iter().map(|&k| (k, k)).collect();
    let mut group = c.benchmark_group(format!("ord_node_capacity/{N}"));

    group.bench_function("lookup", |b| {
        b.iter(|| {
            for k in &keys[..8192] {
                black_box(base.get(black_box(k)));
            }
        });
    });
    group.bench_function("iterate", |b| {
        b.iter(|| {
            for entry in &base {
                black_box(entry);
            }
        });
    });
    group.bench_function("build", |b| {
        b.iter(|| black_box(keys.iter().map(|&k| (k, k)).collect::<Map<N>>()));
    });
    group.bench_function("mutate", |b| {
        let mut map: Map<N> = keys.iter().map(|&k| (k, k)).collect();
        b.iter(|| {
            for &k in &keys[..1024] {
                black_box(map.remove(&k));
                black_box(map.insert(k, k));
            }
        });
    });
    group.bench_function("persistent_update", |b| {
        b.iter(|| {
            for &k in &keys[..128] {
                let mut fork = base.clone();
                fork.insert(k, k + 1);
                black_box(fork);
            }
        });
    });
    group.bench_function("delete", |b| {
        b.iter_batched(
            || base.clone(),
            |mut map| {
                for k in &keys[..keys.len() * 3 / 4] {
                    black_box(map.remove(k));
                }
                black_box(map)
            },
            BatchSize::SmallInput,
        );
    });
    group.bench_function("union", |b| {
        let other: Map<N> = keys.iter().map(|&k| (k + LEN / 2, k)).collect();
        b.iter(|| black_box(base.clone().union(other.clone())));
    });
    group.bench_function("diff_shared", |b| {
        let mut other = base.clone();
        for &k in &keys[..128] {
            other.insert(k, k + 1);
        }
        b.iter(|| {
            for entry in base.diff(&other) {
                black_box(entry);
            }
        });
    });
    group.finish();
}

fn benchmarks(c: &mut Criterion) {
    node_size::<16>(c);
    node_size::<64>(c);
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(30)
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(3));
    targets = benchmarks
}
criterion_main!(benches);
