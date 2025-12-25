//! VeriMantle Gate Benchmarks
//!
//! Per Deep Analysis: "No benchmarks published"
//!
//! Run with: cargo bench

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use std::collections::HashMap;

// Simulated policy check
fn policy_check_simple(input: &str) -> bool {
    !input.contains("DELETE") && !input.contains("DROP")
}

// Simulated CRDT merge
fn crdt_merge(a: &HashMap<String, u64>, b: &HashMap<String, u64>) -> HashMap<String, u64> {
    let mut result = a.clone();
    for (key, &value) in b {
        let entry = result.entry(key.clone()).or_insert(0);
        *entry = (*entry).max(value);
    }
    result
}

// Simulated tokenization
fn tokenize(text: &str) -> Vec<i64> {
    text.split_whitespace()
        .map(|w| w.len() as i64)
        .collect()
}

fn benchmark_policy_check(c: &mut Criterion) {
    let mut group = c.benchmark_group("policy_check");
    
    let inputs = vec![
        ("simple", "transfer 100 to account"),
        ("medium", "SELECT * FROM users WHERE id = 123"),
        ("complex", "DELETE FROM critical_table WHERE 1=1; DROP TABLE users;"),
    ];
    
    for (name, input) in inputs {
        group.throughput(Throughput::Elements(1));
        group.bench_with_input(BenchmarkId::from_parameter(name), &input, |b, &input| {
            b.iter(|| policy_check_simple(black_box(input)));
        });
    }
    
    group.finish();
}

fn benchmark_crdt_merge(c: &mut Criterion) {
    let mut group = c.benchmark_group("crdt_merge");
    
    let sizes = vec![10, 100, 1000];
    
    for size in sizes {
        let a: HashMap<String, u64> = (0..size)
            .map(|i| (format!("key_{}", i), i as u64))
            .collect();
        let b: HashMap<String, u64> = (0..size)
            .map(|i| (format!("key_{}", i), (i * 2) as u64))
            .collect();
        
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &(a.clone(), b.clone()), |bench, (a, b)| {
            bench.iter(|| crdt_merge(black_box(a), black_box(b)));
        });
    }
    
    group.finish();
}

fn benchmark_tokenization(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenization");
    
    let texts = vec![
        ("short", "transfer money"),
        ("medium", "please transfer one hundred dollars to the savings account"),
        ("long", "this is a very long text that contains many words and should take longer to tokenize because it has more content to process through the tokenization pipeline"),
    ];
    
    for (name, text) in texts {
        group.throughput(Throughput::Bytes(text.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(name), &text, |b, &text| {
            b.iter(|| tokenize(black_box(text)));
        });
    }
    
    group.finish();
}

fn benchmark_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");
    group.throughput(Throughput::Elements(10000));
    
    group.bench_function("10k_policy_checks", |b| {
        b.iter(|| {
            for i in 0..10000 {
                policy_check_simple(black_box(&format!("action {}", i)));
            }
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_policy_check,
    benchmark_crdt_merge,
    benchmark_tokenization,
    benchmark_throughput,
);
criterion_main!(benches);
