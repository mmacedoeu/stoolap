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

//! Deterministic collections for blockchain SQL database
//!
//! These collections provide deterministic ordering and hashing
//! for consistent state across blockchain nodes.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

/// Deterministic Map type
/// Uses BTreeMap for deterministic ordering
pub type DetermMap<K, V> = BTreeMap<K, V>;

/// Deterministic Set type
/// Uses BTreeSet for deterministic ordering
pub type DetermSet<T> = BTreeSet<T>;
