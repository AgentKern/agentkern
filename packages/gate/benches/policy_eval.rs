//! Policy Evaluation Benchmarks
//!
//! Per ENGINEERING_STANDARD.md: Fast Path <1ms, Safety Path <20ms

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use agentkern_gate::{GateEngine, Policy, engine::VerificationRequestBuilder};

fn create_test_engine() -> GateEngine {
    let mut engine = GateEngine::new();
    
    // Add a test policy
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        engine.register_policy(Policy {
            id: uuid::Uuid::new_v4(),
            name: "bench-policy".to_string(),
            description: None,
            priority: 100,
            enabled: true,
            jurisdictions: vec!["global".to_string()],
            rules: vec![],
        }).await;
    });
    
    engine
}

fn bench_symbolic_path(c: &mut Criterion) {
    let engine = create_test_engine();
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("symbolic_path_evaluation", |b| {
        b.iter(|| {
            rt.block_on(async {
                let request = VerificationRequestBuilder::new("bench-agent", "read_data")
                    .context("key", serde_json::json!("value"))
                    .build();
                engine.verify(black_box(request)).await
            })
        })
    });
}

fn bench_neural_path(c: &mut Criterion) {
    let engine = create_test_engine();
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("neural_path_evaluation", |b| {
        b.iter(|| {
            rt.block_on(async {
                let request = VerificationRequestBuilder::new("bench-agent", "delete_all_data")
                    .context("dangerous", serde_json::json!(true))
                    .build();
                engine.verify(black_box(request)).await
            })
        })
    });
}

criterion_group!(benches, bench_symbolic_path, bench_neural_path);
criterion_main!(benches);
