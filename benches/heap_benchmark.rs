use std::collections::BinaryHeap;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::seq::SliceRandom;
use rand::thread_rng;

use csheap::{Heap, HeapType};

const SIZES: [usize; 4] = [100, 1_000, 10_000, 100_000];

fn random_vec(size: usize) -> Vec<u32> {
    let mut result: Vec<u32> = (0..size as u32).collect();
    result.shuffle(&mut thread_rng());
    result
}

fn bench_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert_one_by_one");

    for size in SIZES {
        let input = random_vec(size);

        group.bench_with_input(BenchmarkId::new("csheap::Heap", size), &input, |b, input| {
            b.iter(|| {
                let mut heap = Heap::<u32>::new(HeapType::Max);
                for value in input {
                    heap.insert(*value);
                }
                black_box(heap);
            });
        });

        group.bench_with_input(
            BenchmarkId::new("std::BinaryHeap", size),
            &input,
            |b, input| {
                b.iter(|| {
                    let mut heap = BinaryHeap::<u32>::new();
                    for value in input {
                        heap.push(*value);
                    }
                    black_box(heap);
                });
            },
        );
    }

    group.finish();
}

fn bench_from_vec(c: &mut Criterion) {
    let mut group = c.benchmark_group("build_from_existing_vec");

    for size in SIZES {
        let input = random_vec(size);

        group.bench_with_input(BenchmarkId::new("csheap::Heap", size), &input, |b, input| {
            b.iter(|| black_box(Heap::<u32>::from_vec(HeapType::Max, input.clone())));
        });

        group.bench_with_input(
            BenchmarkId::new("std::BinaryHeap", size),
            &input,
            |b, input| {
                b.iter(|| black_box(BinaryHeap::<u32>::from(input.clone())));
            },
        );
    }

    group.finish();
}

fn bench_extract_all(c: &mut Criterion) {
    let mut group = c.benchmark_group("extract_all");

    for size in SIZES {
        let input = random_vec(size);

        group.bench_with_input(BenchmarkId::new("csheap::Heap", size), &input, |b, input| {
            b.iter_batched(
                || Heap::<u32>::from_vec(HeapType::Max, input.clone()),
                |mut heap| {
                    while let Some(value) = heap.extract() {
                        black_box(value);
                    }
                },
                criterion::BatchSize::SmallInput,
            );
        });

        group.bench_with_input(
            BenchmarkId::new("std::BinaryHeap", size),
            &input,
            |b, input| {
                b.iter_batched(
                    || BinaryHeap::<u32>::from(input.clone()),
                    |mut heap| {
                        while let Some(value) = heap.pop() {
                            black_box(value);
                        }
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_insert, bench_from_vec, bench_extract_all);
criterion_main!(benches);
