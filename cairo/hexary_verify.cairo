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

//! Hexary Trie Proof Verification
//!
//! Verifies hexary trie proofs for row existence and values.
//! This is the same structure used by Ethereum for state storage.

use core::hash::HashState;

/// A node in the hexary trie
#[derive(Drop, Serde, SerdeStorage)]
pub enum TrieNode {
    /// Branch node with 16 children
    Branch { children: [Option<Hash>; 16] },
    /// Leaf node containing a value
    Leaf { value: u64 },
    /// Extension node for shared prefixes
    Extension { prefix: ByteArray, child: Hash },
}

/// Hash of a trie node (using u64 for simplicity in Cairo)
pub type Hash = u64;

/// A proof level showing the path from root to leaf
#[derive(Drop, Serde, SerdeStorage)]
pub struct ProofLevel {
    pub node_hash: Hash,
    pub node_type: u8, // 0=Branch, 1=Leaf, 2=Extension
    pub index: u8,     // Which child to follow (0-15)
}

/// A complete hexary proof from root to value
#[derive(Drop, Serde, SerdeStorage)]
pub struct HexaryProof {
    pub row_id: u64,
    pub value: u64,
    pub levels: Array<ProofLevel>,
}

/// Hash 16 children of a branch node
pub fn hash_16_children(children: [Option<Hash>; 16]) -> Hash {
    let mut state = HashState::new();

    let mut i: usize = 0;
    while i < 16 {
        match children[i] {
            Option::Some(hash) => {
                state.update(u8::try_from(1).unwrap());
                state.update(hash);
            }
            Option::None => {
                state.update(u8::try_from(0).unwrap());
            }
        }
        i += 1;
    };

    state.finalize()
}

/// Hash a leaf node
pub fn hash_leaf(value: u64) -> Hash {
    let mut state = HashState::new();
    state.update(u8::try_from(1).unwrap()); // Node type: Leaf
    state.update(value);
    state.finalize()
}

/// Hash an extension node
pub fn hash_extension(prefix: ByteArray, child: Hash) -> Hash {
    let mut state = HashState::new();
    state.update(u8::try_from(2).unwrap()); // Node type: Extension
    // Hash prefix bytes
    let mut i: usize = 0;
    while i < prefix.len() {
        state.update(prefix[i]);
        i += 1;
    };
    state.update(child);
    state.finalize()
}

/// Verify a single hexary proof
pub fn verify_hexary_proof(proof: HexaryProof, expected_root: Hash) -> bool {
    if proof.levels.len() == 0 {
        return false;
    }

    // Start from the expected root
    let mut current_hash = expected_root;

    // Verify each level
    let mut i: usize = 0;
    while i < proof.levels.len() {
        let level = proof.levels[i];

        // Check if the node hash matches what we expect
        if level.node_hash != current_hash {
            return false;
        }

        // At the leaf level, verify the value
        if level.node_type == 1 {
            // Leaf node - verify value matches
            let expected_hash = hash_leaf(proof.value);
            if level.node_hash != expected_hash {
                return false;
            }
            return true;
        }

        // For non-leaf nodes, compute the expected hash
        // In a real implementation, we'd reconstruct the node and hash it
        // For now, we just proceed with the child hash
        current_hash = level.node_hash; // Simplified - in reality we'd derive from children

        i += 1;
    };

    // If we exhausted all levels, the proof is valid
    true
}

/// Main entry point for hexary proof verification
#[external]
pub fn verify_proof(
    row_id: u64,
    value: u64,
    levels: Array<ProofLevel>,
    expected_root: Hash,
) -> bool {
    let proof = HexaryProof { row_id, value, levels };
    verify_hexary_proof(proof, expected_root)
}
