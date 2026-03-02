// stwo-bench/stwo_proof.rs
// Real STWO benchmarks - uses local stwo-cairo with adapter module

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use stwo_cairo_adapter::ProverInput;
use stwo_cairo_prover::prover::{ProverParameters, ChannelHash};
use stwo::core::vcs_lifted::blake2_merkle::Blake2sMerkleChannel;
use stwo::core::pcs::PcsConfig;
use stwo::core::fri::FriConfig;
use cairo_air::PreProcessedTraceVariant;
use cairo_air::CairoProofForRustVerifier;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sMerkleHasher;

// Path to stwo-cairo test data
const STWO_CAIRO_PATH: &str = "/home/mmacedoeu/_w/crypto/stwo-cairo/stwo_cairo_prover/test_data";

fn create_default_prover_params() -> ProverParameters {
    ProverParameters {
        channel_hash: ChannelHash::Blake2s,
        channel_salt: 0,
        pcs_config: PcsConfig {
            pow_bits: 26,
            fri_config: FriConfig {
                log_last_layer_degree_bound: 0,
                log_blowup_factor: 1,
                n_queries: 70,
                line_fold_step: 1,
            },
            lifting_log_size: None,
        },
        preprocessed_trace: PreProcessedTraceVariant::Canonical,
        store_polynomials_coefficients: false,
        include_all_preprocessed_columns: false,
    }
}

pub fn bench_real_proof_generation_merkle_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("stark_real_proof_generation_merkle_batch");

    // Use all_builtins test case which has prover_input.json
    let prover_input_path = format!("{}/test_prove_verify_all_builtins/prover_input.json", STWO_CAIRO_PATH);

    // Load ProverInput from JSON (outside timing)
    let input_json = std::fs::read_to_string(&prover_input_path)
        .expect("Failed to read prover input");
    let prover_input: ProverInput = serde_json::from_str(&input_json)
        .expect("Failed to parse prover input");

    let params = create_default_prover_params();

    for size in [1].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &_size| {
            b.iter(|| {
                // Actually call prove_cairo with Blake2sMerkleChannel
                let result = stwo_cairo_prover::prover::prove_cairo::<Blake2sMerkleChannel>(
                    prover_input.clone(),
                    params,
                );
                let _ = std::hint::black_box(result);
            });
        });
    }
    group.finish();
}

pub fn bench_real_proof_verification_merkle_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("stark_real_proof_verification_merkle_batch");

    // Generate proof first (outside timing)
    let prover_input_path = format!("{}/test_prove_verify_all_builtins/prover_input.json", STWO_CAIRO_PATH);
    let input_json = std::fs::read_to_string(&prover_input_path)
        .expect("Failed to read prover input");
    let prover_input: ProverInput = serde_json::from_str(&input_json)
        .expect("Failed to parse prover input");
    let params = create_default_prover_params();

    let proof = stwo_cairo_prover::prover::prove_cairo::<Blake2sMerkleChannel>(
        prover_input.clone(),
        params,
    ).expect("Failed to generate proof");

    // Convert to verifier-compatible format
    let proof_for_verifier: CairoProofForRustVerifier<Blake2sMerkleHasher> = proof.into();

    for size in [1].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &_size| {
            b.iter(|| {
                // Actually verify the proof using verify_cairo
                let result = cairo_air::verifier::verify_cairo::<Blake2sMerkleChannel>(
                    proof_for_verifier.clone(),
                );
                let _ = std::hint::black_box(result);
            });
        });
    }
    group.finish();
}

pub fn bench_real_proof_generation_hexary_verify(c: &mut Criterion) {
    let mut group = c.benchmark_group("stark_real_proof_generation_hexary_verify");

    // Use all_builtins test case
    let prover_input_path = format!("{}/test_prove_verify_all_builtins/prover_input.json", STWO_CAIRO_PATH);

    // Load ProverInput from JSON (outside timing)
    let input_json = std::fs::read_to_string(&prover_input_path)
        .expect("Failed to read prover input");
    let prover_input: ProverInput = serde_json::from_str(&input_json)
        .expect("Failed to parse prover input");

    let params = create_default_prover_params();

    for size in [1].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &_size| {
            b.iter(|| {
                let result = stwo_cairo_prover::prover::prove_cairo::<Blake2sMerkleChannel>(
                    prover_input.clone(),
                    params,
                );
                let _ = std::hint::black_box(result);
            });
        });
    }
    group.finish();
}

pub fn bench_real_proof_verification_hexary_verify(c: &mut Criterion) {
    let mut group = c.benchmark_group("stark_real_proof_verification_hexary_verify");

    // Generate proof first (outside timing)
    let prover_input_path = format!("{}/test_prove_verify_all_builtins/prover_input.json", STWO_CAIRO_PATH);
    let input_json = std::fs::read_to_string(&prover_input_path)
        .expect("Failed to read prover input");
    let prover_input: ProverInput = serde_json::from_str(&input_json)
        .expect("Failed to parse prover input");
    let params = create_default_prover_params();

    let proof = stwo_cairo_prover::prover::prove_cairo::<Blake2sMerkleChannel>(
        prover_input.clone(),
        params,
    ).expect("Failed to generate proof");

    let proof_for_verifier: CairoProofForRustVerifier<Blake2sMerkleHasher> = proof.into();

    for size in [1].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &_size| {
            b.iter(|| {
                let result = cairo_air::verifier::verify_cairo::<Blake2sMerkleChannel>(
                    proof_for_verifier.clone(),
                );
                let _ = std::hint::black_box(result);
            });
        });
    }
    group.finish();
}

pub fn bench_real_proof_generation_state_transition(c: &mut Criterion) {
    let mut group = c.benchmark_group("stark_real_proof_generation_state_transition");

    // Use opcode components test case
    let prover_input_path = format!("{}/test_prove_verify_all_opcode_components/prover_input.json", STWO_CAIRO_PATH);

    // Load ProverInput from JSON (outside timing)
    let input_json = std::fs::read_to_string(&prover_input_path)
        .expect("Failed to read prover input");
    let prover_input: ProverInput = serde_json::from_str(&input_json)
        .expect("Failed to parse prover input");

    let params = create_default_prover_params();

    for size in [1].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &_size| {
            b.iter(|| {
                let result = stwo_cairo_prover::prover::prove_cairo::<Blake2sMerkleChannel>(
                    prover_input.clone(),
                    params,
                );
                let _ = std::hint::black_box(result);
            });
        });
    }
    group.finish();
}

pub fn bench_real_proof_verification_state_transition(c: &mut Criterion) {
    let mut group = c.benchmark_group("stark_real_proof_verification_state_transition");

    // Generate proof first (outside timing)
    let prover_input_path = format!("{}/test_prove_verify_all_opcode_components/prover_input.json", STWO_CAIRO_PATH);
    let input_json = std::fs::read_to_string(&prover_input_path)
        .expect("Failed to read prover input");
    let prover_input: ProverInput = serde_json::from_str(&input_json)
        .expect("Failed to parse prover input");
    let params = create_default_prover_params();

    let proof = stwo_cairo_prover::prover::prove_cairo::<Blake2sMerkleChannel>(
        prover_input.clone(),
        params,
    ).expect("Failed to generate proof");

    let proof_for_verifier: CairoProofForRustVerifier<Blake2sMerkleHasher> = proof.into();

    for size in [1].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &_size| {
            b.iter(|| {
                let result = cairo_air::verifier::verify_cairo::<Blake2sMerkleChannel>(
                    proof_for_verifier.clone(),
                );
                let _ = std::hint::black_box(result);
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
