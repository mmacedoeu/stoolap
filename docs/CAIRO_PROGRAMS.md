# Cairo Programs Documentation

This document describes the Cairo programs bundled with Stoolap for zero-knowledge proof generation.

## Overview

Stoolap includes three core Cairo programs that implement verification logic for:

1. **State Transitions** - Verify blockchain state changes
2. **Hexary Proofs** - Verify trie membership proofs
3. **Batch Verification** - Efficiently verify multiple proofs

These programs can be executed and proven using STWO to generate STARK proofs that can be verified on-chain.

## Programs

### 1. state_transition.cairo

Verifies that applying a sequence of database operations to a previous state root results in the expected new state root.

**Type**: `Operation`
```cairo
pub enum Operation {
    Insert { row_id: u64, row_hash: u64 },
    Update { row_id: u64, old_hash: u64, new_hash: u64 },
    Delete { row_id: u64, old_hash: u64 },
}
```

**Functions**:
- `hash_operation(operation: Operation) -> u64` - Hash an operation
- `apply_operation(state_hash: u64, operation: Operation) -> u64` - Apply operation to state
- `verify_root(prev_root: u64, operations: Array<Operation>, new_root: u64) -> bool` - Verify state transition

**External Entry Point**:
```cairo
#[external]
pub fn verify_state_transition(
    prev_root: u64,
    operations: Array<Operation>,
    new_root: u64,
) -> bool
```

**Use Cases**:
- Validating blockchain state transitions
- Verifying transaction batches
- Proving correct state updates

### 2. hexary_verify.cairo

Verifies hexary trie (Ethereum-style) proofs for row existence and values.

**Types**:
```cairo
pub struct ProofLevel {
    pub node_hash: Hash,
    pub node_type: u8,  // 0=Branch, 1=Leaf, 2=Extension
    pub index: u8,      // Which child to follow (0-15)
}

pub struct HexaryProof {
    pub row_id: u64,
    pub value: u64,
    pub levels: Array<ProofLevel>,
}
```

**Functions**:
- `hash_16_children(children: [Option<Hash>; 16]) -> Hash` - Hash branch node children
- `hash_leaf(value: u64) -> Hash` - Hash a leaf node
- `hash_extension(prefix: ByteArray, child: Hash) -> Hash` - Hash an extension node
- `verify_hexary_proof(proof: HexaryProof, expected_root: Hash) -> bool` - Verify a proof

**External Entry Point**:
```cairo
#[external]
pub fn verify_proof(
    row_id: u64,
    value: u64,
    levels: Array<ProofLevel>,
    expected_root: Hash,
) -> bool
```

**Use Cases**:
- Verifying single row existence proofs
- Lightweight SPV proofs
- State verification

### 3. merkle_batch.cairo

Efficiently verifies multiple hexary proofs in a single execution.

**Types**:
```cairo
pub struct SingleProof {
    pub row_id: u64,
    pub value: u64,
    pub proof_hash: u64,  // Pre-computed hash of the proof path
}

pub struct BatchResult {
    pub valid: bool,
    pub count: u64,
}
```

**Functions**:
- `verify_single_proof(proof: SingleProof, root: u64) -> bool` - Verify one proof
- `batch_verify(proofs: Array<SingleProof>, expected_root: u64) -> BatchResult` - Verify all proofs
- `batch_verify_strict(proofs: Array<SingleProof>, expected_root: u64) -> BatchResult` - Verify with early exit

**External Entry Points**:
```cairo
#[external]
pub fn verify_proofs_batch(
    proofs: Array<SingleProof>,
    expected_root: u64,
    strict: bool,
) -> BatchResult

#[external]
pub fn verify_single(
    row_id: u64,
    value: u64,
    proof_hash: u64,
    expected_root: u64,
) -> bool

#[external]
pub fn count_valid_proofs(
    proofs: Array<SingleProof>,
    expected_root: u64,
) -> u64
```

**Use Cases**:
- Batch proof verification (gas efficient)
- Aggregating multiple proofs
- Counting valid proofs in a set

## Rust Integration

### Register Bundled Programs

```rust
use stoolap::zk::{CairoProgramRegistry, register_bundled_programs};

let mut registry = CairoProgramRegistry::new();
register_bundled_programs(&mut registry)?;
```

### Get Bundled Program

```rust
use stoolap::zk::get_bundled_program;

let program = get_bundled_program("state_transition")
    .expect("bundled program not found");
```

### Check Program Hash

```rust
use stoolap::zk::{is_bundled_program, get_bundled_program_name, STATE_TRANSITION_HASH};

if is_bundled_program(&hash) {
    let name = get_bundled_program_name(&hash);
    println!("Program: {}", name.unwrap());
}
```

### Generate Proof

```rust
use stoolap::zk::{STWOProver, get_bundled_program};

let program = get_bundled_program("state_transition")?;
let prover = STWOProver::new();

// Prepare inputs (encoded operation data)
let inputs = encode_operations(&operations)?;

// Generate proof
let proof = prover.prove(&program, &inputs)?;
```

### Verify Proof

```rust
use stoolap::zk::{STWOProver, StarkProof};

let prover = STWOProver::new();
let is_valid = prover.verify(&proof, &expected_outputs)?;
```

## Program Hashes

| Program | Hash Constant |
|---------|---------------|
| state_transition | `STATE_TRANSITION_HASH` |
| hexary_verify | `HEXARY_VERIFY_HASH` |
| merkle_batch | `MERKLE_BATCH_HASH` |

## Input Encoding

Inputs to Cairo programs must be encoded according to the program's expected format:

1. **state_transition**: Encode as (prev_root, operation_count, operations...)
2. **hexary_verify**: Encode as (row_id, value, level_count, levels...)
3. **merkle_batch**: Encode as (proof_count, proofs...)

See individual program documentation for exact encoding specifications.

## Compilation

These programs are written in Cairo 2.0 syntax. To rebuild the CASM bytecode:

```bash
# Compile to Sierra
cairo-compile cairo/state_transition.cairo --sierra

# Compile Sierra to CASM
sierra-compile state_transition.sierra
```

The resulting CASM should replace the stub bytecode in `src/zk/bundled.rs`.

## Security Considerations

1. **Hash Collisions**: Programs use `HashState` for cryptographic hashing
2. **Input Validation**: All inputs should be validated before proof generation
3. **Proof Size Limits**: Prover enforces maximum proof sizes (default 500KB)
4. **Program Allowlist**: Only registered programs can be used for proof generation

## Future Enhancements

- Add compression algorithms to reduce proof size
- Implement recursive proof composition
- Add support for custom Cairo programs
- Optimize gas costs for on-chain verification

## References

- [Cairo Documentation](https://book.cairo-lang.org/)
- [STWO Repository](https://github.com/starkware-libs/stwo)
- [RFC-0201: STWO and Cairo Integration](/missions/archived/0201-00-stwo-cairo-integration.md)
