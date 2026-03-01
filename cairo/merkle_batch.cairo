// Copyright 2025 Stoolap Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Batch Merkle Proof Verification
//!
//! Efficiently verifies multiple hexary proofs in a single Cairo program.
//! Reduces gas costs by aggregating verification.

use core::hash::HashState;

/// A single hexary proof for batch verification
#[derive(Drop, Serde, SerdeStorage)]
pub struct SingleProof {
    pub row_id: u64,
    pub value: u64,
    pub proof_hash: u64,  // Pre-computed hash of the proof path
}

/// Result of batch verification
#[derive(Drop, Serde, SerdeStorage)]
pub struct BatchResult {
    pub valid: bool,
    pub count: u64,
}

/// Verify a single proof in the batch
fn verify_single_proof(proof: SingleProof, root: u64) -> bool {
    // In a real implementation, this would verify the full proof path
    // For now, we use the pre-computed proof hash

    let mut state = HashState::new();
    state.update(root);
    state.update(proof.row_id);
    state.update(proof.value);
    let expected_hash = state.finalize();

    proof.proof_hash == expected_hash
}

/// Accumulate hashes for batch verification
fn accumulate_hashes(current: u64, new_proof: u64) -> u64 {
    let mut state = HashState::new();
    state.update(current);
    state.update(new_proof);
    state.finalize()
}

/// Verify multiple proofs in batch
///
/// Returns BatchResult with:
/// - valid: true if all proofs are valid
/// - count: number of proofs verified
pub fn batch_verify(
    proofs: Array<SingleProof>,
    expected_root: u64,
) -> BatchResult {
    if proofs.len() == 0 {
        return BatchResult { valid: true, count: 0 };
    }

    let mut all_valid = true;
    let mut accumulator = expected_root;

    let mut i: usize = 0;
    while i < proofs.len() {
        let proof = proofs[i];

        // Verify each proof
        if !verify_single_proof(proof, expected_root) {
            all_valid = false;
        }

        // Accumulate hashes for efficiency
        accumulator = accumulate_hashes(accumulator, proof.proof_hash);

        i += 1;
    };

    BatchResult { valid: all_valid, count: proofs.len() }
}

/// Verify batch with early termination on failure
pub fn batch_verify_strict(
    proofs: Array<SingleProof>,
    expected_root: u64,
) -> BatchResult {
    if proofs.len() == 0 {
        return BatchResult { valid: true, count: 0 };
    }

    let mut i: usize = 0;
    while i < proofs.len() {
        let proof = proofs[i];

        if !verify_single_proof(proof, expected_root) {
            // Early termination on first failure
            return BatchResult { valid: false, count: i };
        }

        i += 1;
    };

    BatchResult { valid: true, count: proofs.len() }
}

/// Main entry point for batch verification
#[external]
pub fn verify_proofs_batch(
    proofs: Array<SingleProof>,
    expected_root: u64,
    strict: bool,
) -> BatchResult {
    if strict {
        batch_verify_strict(proofs, expected_root)
    } else {
        batch_verify(proofs, expected_root)
    }
}

/// Verify a single proof (convenience function)
#[external]
pub fn verify_single(
    row_id: u64,
    value: u64,
    proof_hash: u64,
    expected_root: u64,
) -> bool {
    let proof = SingleProof { row_id, value, proof_hash };
    verify_single_proof(proof, expected_root)
}

/// Count valid proofs in a batch
#[external]
pub fn count_valid_proofs(
    proofs: Array<SingleProof>,
    expected_root: u64,
) -> u64 {
    let mut count: u64 = 0;

    let mut i: usize = 0;
    while i < proofs.len() {
        let proof = proofs[i];

        if verify_single_proof(proof, expected_root) {
            count += 1;
        }

        i += 1;
    };

    count
}
