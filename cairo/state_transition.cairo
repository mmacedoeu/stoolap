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

//! State Transition Verification
//!
//! Verifies that applying operations to a previous state root results
//! in the expected new state root. This is used for validating blockchain
//! state transitions in ZK proofs.

use core::hash::HashState;

mod HashStateTrait {
    fn update_state(ref self: HashState, operation: Operation) -> HashState;
}

/// Represents a database operation
#[derive(Drop, Serde, SerdeStorage)]
pub enum Operation {
    Insert { row_id: u64, row_hash: u64 },
    Update { row_id: u64, old_hash: u64, new_hash: u64 },
    Delete { row_id: u64, old_hash: u64 },
}

/// Hash an operation into a state hash value
pub fn hash_operation(operation: Operation) -> u64 {
    match operation {
        Operation::Insert { row_id, row_hash } => {
            let mut state = HashState::new();
            state.update(u8::try_from(1).unwrap()); // Operation type
            state.update(row_id);
            state.update(row_hash);
            state.finalize()
        }
        Operation::Update { row_id, old_hash, new_hash } => {
            let mut state = HashState::new();
            state.update(u8::try_from(2).unwrap()); // Operation type
            state.update(row_id);
            state.update(old_hash);
            state.update(new_hash);
            state.finalize()
        }
        Operation::Delete { row_id, old_hash } => {
            let mut state = HashState::new();
            state.update(u8::try_from(3).unwrap()); // Operation type
            state.update(row_id);
            state.update(old_hash);
            state.finalize()
        }
    }
}

/// Apply an operation to a state hash
pub fn apply_operation(mut state_hash: u64, operation: Operation) -> u64 {
    // Combine current state with operation hash
    let operation_hash = hash_operation(operation);
    let mut state = HashState::new();
    state.update(state_hash);
    state.update(operation_hash);
    state.finalize()
}

/// Verify that applying operations to prev_root results in new_root
pub fn verify_root(
    prev_root: u64,
    operations: Array<Operation>,
    new_root: u64,
) -> bool {
    let mut current_root = prev_root;

    let mut i: usize = 0;
    while i < operations.len() {
        current_root = apply_operation(current_root, operations[i]);
        i += 1;
    };

    current_root == new_root
}

/// Main entry point for state transition verification
#[external]
pub fn verify_state_transition(
    prev_root: u64,
    operations: Array<Operation>,
    new_root: u64,
) -> bool {
    verify_root(prev_root, operations, new_root)
}
