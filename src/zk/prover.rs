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

//! STWO prover wrapper
//!
//! Provides Rust interface to STWO's Circle STARK prover/verifier.

use std::time::Duration;

/// Configuration for STWO prover
#[derive(Debug, Clone)]
pub struct ProverConfig {
    /// Maximum proof size in bytes
    pub max_proof_size: usize,

    /// Timeout for proof generation
    pub timeout: Duration,

    /// Number of threads for parallel proof generation
    pub num_threads: usize,
}

impl Default for ProverConfig {
    fn default() -> Self {
        Self {
            max_proof_size: 500 * 1024, // 500 KB
            timeout: Duration::from_secs(30),
            num_threads: rayon::current_num_threads().max(1),
        }
    }
}

/// STWO prover for generating STARK proofs
///
/// This is a placeholder wrapper. The actual STWO integration
/// will be implemented in future missions.
#[derive(Debug, Clone)]
pub struct STWOProver {
    config: ProverConfig,
}

impl STWOProver {
    /// Create a new STWO prover with default configuration
    pub fn new() -> Self {
        Self {
            config: ProverConfig::default(),
        }
    }

    /// Create a new STWO prover with custom configuration
    pub fn with_config(config: ProverConfig) -> Self {
        Self { config }
    }

    /// Get the prover configuration
    pub fn config(&self) -> &ProverConfig {
        &self.config
    }

    /// Set the maximum proof size
    pub fn with_max_proof_size(mut self, size: usize) -> Self {
        self.config.max_proof_size = size;
        self
    }

    /// Set the proof generation timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
        self
    }
}

impl Default for STWOProver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prover_creation() {
        let prover = STWOProver::new();
        assert_eq!(prover.config().max_proof_size, 500 * 1024);
    }

    #[test]
    fn test_prover_with_config() {
        let config = ProverConfig {
            max_proof_size: 1024,
            timeout: Duration::from_secs(10),
            num_threads: 4,
        };
        let prover = STWOProver::with_config(config.clone());
        assert_eq!(prover.config().max_proof_size, 1024);
    }

    #[test]
    fn test_prover_builder_pattern() {
        let prover = STWOProver::new()
            .with_max_proof_size(2048)
            .with_timeout(Duration::from_secs(60));

        assert_eq!(prover.config().max_proof_size, 2048);
        assert_eq!(prover.config().timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_default_prover() {
        let prover = STWOProver::default();
        assert_eq!(prover.config().max_proof_size, 500 * 1024);
    }
}
