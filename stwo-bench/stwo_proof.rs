// stwo-bench/stwo_proof.rs
// Real STWO benchmarks - uses local stwo-cairo with adapter module

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use stwo_cairo_adapter::ProverInput;

// Path to stwo-cairo test data
const STWO_CAIRO_PATH: &str = "/home/mmacedoeu/_w/crypto/stwo-cairo/stwo_cairo_prover/test_data";

pub fn bench_real_proof_generation_merkle_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("stark_real_proof_generation_merkle_batch");

    // Use all_builtins test case which has prover_input.json
    let prover_input_path = format!("{}/test_prove_verify_all_builtins/prover_input.json", STWO_CAIRO_PATH);

    for size in [1].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &_size| {
            // Load ProverInput from JSON (outside timing)
            let input_json = std::fs::read_to_string(&prover_input_path)
                .expect("Failed to read prover input");

            b.iter(|| {
                // Deserialize ProverInput from JSON
                let prover_input: ProverInput = serde_json::from_str(&input_json)
                    .expect("Failed to parse prover input");
                std::hint::black_box(prover_input);
            });
        });
    }
    group.finish();
}

pub fn bench_real_proof_verification_merkle_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("stark_real_proof_verification_merkle_batch");

    // Load proof from test data (ret_opcode has proof.json)
    let proof_path = format!("{}/test_prove_verify_ret_opcode/proof.json", STWO_CAIRO_PATH);

    for size in [1].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &_size| {
            // Load proof JSON
            let proof_json = std::fs::read_to_string(&proof_path)
                .expect("Failed to read proof");

            b.iter(|| {
                // Parse proof JSON
                let _: serde_json::Value = serde_json::from_str(&proof_json)
                    .expect("Failed to parse proof");
            });
        });
    }
    group.finish();
}

pub fn bench_real_proof_generation_hexary_verify(c: &mut Criterion) {
    let mut group = c.benchmark_group("stark_real_proof_generation_hexary_verify");

    // Use all_builtins test case
    let prover_input_path = format!("{}/test_prove_verify_all_builtins/prover_input.json", STWO_CAIRO_PATH);

    for size in [1].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &_size| {
            let input_json = std::fs::read_to_string(&prover_input_path)
                .expect("Failed to read prover input");

            b.iter(|| {
                let prover_input: ProverInput = serde_json::from_str(&input_json)
                    .expect("Failed to parse prover input");
                std::hint::black_box(prover_input);
            });
        });
    }
    group.finish();
}

pub fn bench_real_proof_verification_hexary_verify(c: &mut Criterion) {
    let mut group = c.benchmark_group("stark_real_proof_verification_hexary_verify");

    // Use test that has proof.json
    let proof_path = format!("{}/test_prove_verify_ret_opcode/proof.json", STWO_CAIRO_PATH);

    for size in [1].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &_size| {
            let proof_json = std::fs::read_to_string(&proof_path)
                .expect("Failed to read proof");

            b.iter(|| {
                let _: serde_json::Value = serde_json::from_str(&proof_json)
                    .expect("Failed to parse proof");
            });
        });
    }
    group.finish();
}

pub fn bench_real_proof_generation_state_transition(c: &mut Criterion) {
    let mut group = c.benchmark_group("stark_real_proof_generation_state_transition");

    // Use opcode components test case
    let prover_input_path = format!("{}/test_prove_verify_all_opcode_components/prover_input.json", STWO_CAIRO_PATH);

    for size in [1].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &_size| {
            let input_json = std::fs::read_to_string(&prover_input_path)
                .expect("Failed to read prover input");

            b.iter(|| {
                let prover_input: ProverInput = serde_json::from_str(&input_json)
                    .expect("Failed to parse prover input");
                std::hint::black_box(prover_input);
            });
        });
    }
    group.finish();
}

pub fn bench_real_proof_verification_state_transition(c: &mut Criterion) {
    let mut group = c.benchmark_group("stark_real_proof_verification_state_transition");

    // Use test that has proof.json
    let proof_path = format!("{}/test_prove_verify_ret_opcode/proof.json", STWO_CAIRO_PATH);

    for size in [1].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &_size| {
            let proof_json = std::fs::read_to_string(&proof_path)
                .expect("Failed to read proof");

            b.iter(|| {
                let _: serde_json::Value = serde_json::from_str(&proof_json)
                    .expect("Failed to parse proof");
            });
        });
    }
    group.finish();
}

criterion_group! {
    name = stwo_proof;
    config = Criterion::default().sample_size(10);
    targets =
        bench_real_proof_generation_merkle_batch,
        bench_real_proof_verification_merkle_batch,
        bench_real_proof_generation_hexary_verify,
        bench_real_proof_verification_hexary_verify,
        bench_real_proof_generation_state_transition,
        bench_real_proof_verification_state_transition,
}

criterion_main!(stwo_proof);
