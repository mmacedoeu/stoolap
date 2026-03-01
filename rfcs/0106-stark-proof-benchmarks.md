# RFC-0106: STARK Proof Benchmarks with Real STWO

## Status
Draft

## Summary

Add benchmark infrastructure for STARK proof generation and verification using real STWO prover, with comparison to current mock implementation. Duplicated benchmarks for both mock and real (not feature-gated).

## Motivation

The current `STWOProver::prove()` method uses `generate_mock_proof()` which creates fake proof data. This doesn't reflect real STWO performance, making it impossible to:
- Measure actual proving time
- Compare mock vs real overhead
- Plan production infrastructure

## Specification

### Architecture

```
benches/stark_proof.rs
├── bench_mock_proof_generation      # Using generate_mock_proof() - always available
├── bench_real_proof_generation      # Using stwo-cairo-prover - requires zk feature
├── bench_mock_proof_verification    # Verify mock proofs - always available
└── bench_real_proof_verification   # Verify real STWO proofs - requires zk feature
```

### Dependencies

The `zk` feature already exists. Add `stwo-cairo-prover` as part of zk:

```toml
# Already exists:
# stwo = { version = "2.1", optional = true }

# Add:
stwo-cairo-prover = "1.1"
```

### No Feature Flag - Duplicated Benchmarks

Instead of feature-gated code, duplicate benchmark functions:
- Mock benchmarks: Always compile (no STWO dependency)
- Real benchmarks: Require `zk` feature

This allows running both in CI when zk is enabled.

### Benchmark Parameters

- **Batch sizes:** 10, 100, 1000 rows
- **Cairo programs:**
  - `hexary_verify.cairo` - Single proof verification
  - `merkle_batch.cairo` - Batch verification
  - `state_transition.cairo` - State transitions

### Cairo Program Compilation

Create `cairo/build.rs` to compile `.cairo` → `.casm` at build time:
```rust
fn main() {
    // Compile Cairo programs to CASM
    compile_cairo("hexary_verify");
    compile_cairo("merkle_batch");
    compile_cairo("state_transition");
}
```

### Prover Integration

Two separate methods in `STWOProver`:

```rust
// Existing - always available
fn generate_mock_proof(&self, program: &CairoProgram, inputs: &[u8]) -> Result<StarkProof, ProverError> {
    // Current implementation - creates fake proof
}

// New - requires zk feature (for real benchmarks)
#[cfg(feature = "zk")]
fn generate_real_proof(&self, program: &CairoProgram, inputs: &[u8]) -> Result<StarkProof, ProverError> {
    use stwo_cairo_prover::CairoProver;

    let prover = CairoProver::new(self.config.clone());
    prover.prove(program.casm(), inputs)
}
```

### Benchmark Implementation

```rust
// Mock - always available
fn bench_mock_proof_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("stark_mock_proof_generation");

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let prover = STWOProver::new();
            let inputs = generate_batch_inputs(size);

            b.iter(|| {
                prover.generate_mock_proof(&program, &inputs);
            });
        });
    }
}

// Real - requires zk feature
#[cfg(feature = "zk")]
fn bench_real_proof_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("stark_real_proof_generation");

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let prover = STWOProver::new();
            let inputs = generate_batch_inputs(size);

            b.iter(|| {
                prover.generate_real_proof(&program, &inputs);
            });
        });
    }
}
```

## Rationale

### Why Duplicated Benchmarks?

1. **No conditional compilation** - Clear, explicit code
2. **CI friendly** - Both run when zk is enabled
3. **Easy comparison** - Same structure, different implementation
4. **No runtime feature detection** - Compile-time selection

### Alternatives Considered

| Approach | Pros | Cons |
|----------|------|------|
| A: Feature-gated functions | Single code path | Complex, harder to compare |
| B: Runtime detection | Single binary | Slower, added complexity |
| C: Duplicated (chosen) | Clear, comparable | Slightly more code |

## Implementation

### Files to Create

1. `benches/stark_proof.rs` - Benchmark suite with duplicated benchmarks
2. `cairo/build.rs` - Cairo compilation

### Files to Modify

1. `Cargo.toml` - Add stwo-cairo-prover dependency
2. `src/zk/prover.rs` - Add `generate_real_proof()` method

### Cairo Programs

| Program | Hash | Purpose |
|--------|------|---------|
| `hexary_verify.cairo` | HEXARY_VERIFY_HASH | Verify hexary proofs |
| `merkle_batch.cairo` | MERKLE_BATCH_HASH | Batch verification |
| `state_transition.cairo` | STATE_TRANSITION_HASH | State transitions |

## Testing Requirements

- [ ] Mock benchmarks compile without zk feature
- [ ] Real benchmarks compile with `--features zk`
- [ ] Both can run: `cargo bench --features zk`
- [ ] Mock shows ~0ms (instant)
- [ ] Real shows actual proving time

## Performance Expectations

- Mock generation: ~0ms (instant)
- Real generation: varies by program size (expected 100ms-10s)
- Mock verification: ~0ms (instant)
- Real verification: expected 10ms-1s

## Related Use Cases

- [STARK Proof Benchmarks](../docs/use-cases/stark-proof-benchmarks.md)

## Related RFCs

- [RFC-0201: STWO and Cairo Integration](./0201-stwo-cairo-integration.md)
- [RFC-0202: Compressed Proofs](./0202-compressed-proofs.md)
