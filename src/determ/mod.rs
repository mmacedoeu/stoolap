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

//! Deterministic types for blockchain SQL database
//!
//! This module provides deterministic types that:
//! - Use no Arc/pointers for predictable memory layout
//! - Support Merkle hashing for consistent state across nodes
//! - Are fully serializable for network transmission
//! - Have deterministic ordering for consensus

pub mod collections;
pub mod row;
pub mod value;

#[cfg(test)]
mod tests;

pub use collections::{DetermMap, DetermSet};
pub use row::DetermRow;
pub use value::DetermValue;
