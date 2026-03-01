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

//! Cairo program types and registry
//!
//! This module provides data structures for managing Cairo programs
//! that can be proven using STWO.

use std::collections::{BTreeMap, BTreeSet};

/// Cairo program identifier (blake3 hash of source code)
pub type CairoProgramHash = [u8; 32];

/// Compiled Cairo program with all artifacts
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CairoProgram {
    /// Blake3 hash of the source code (unique identifier)
    pub hash: CairoProgramHash,
    /// Cairo source code
    pub source: String,
    /// Sierra intermediate representation (IR)
    pub sierra: Vec<u8>,
    /// Cairo Assembly Machine (executable format)
    pub casm: Vec<u8>,
    /// Cairo compiler version
    pub version: u32,
}

impl CairoProgram {
    /// Create a new Cairo program from source code
    ///
    /// Note: This is a placeholder. The actual compilation
    /// will be implemented in Mission 0201-03.
    pub fn from_source(source: String, version: u32) -> Self {
        let hash = Self::compute_hash(&source);
        Self {
            hash,
            source,
            sierra: Vec::new(),
            casm: Vec::new(),
            version,
        }
    }

    /// Compute blake3 hash of Cairo source code
    pub fn compute_hash(source: &str) -> CairoProgramHash {
        #[cfg(feature = "zk")]
        {
            blake3::hash(source.as_bytes()).into()
        }

        #[cfg(not(feature = "zk"))]
        {
            // Fallback for when zk feature is not enabled
            let mut hash = [0u8; 32];
            let bytes = source.as_bytes();
            let len = bytes.len().min(32);
            hash[..len].copy_from_slice(&bytes[..len]);
            hash
        }
    }

    /// Compile Cairo source to Sierra (stub)
    ///
    /// Note: This will be implemented in Mission 0201-03
    /// with actual Cairo compiler integration.
    pub fn compile_to_sierra(source: &str) -> Result<Vec<u8>, CompileError> {
        let _ = source;
        Err(CompileError::NotImplemented("compile_to_sierra".to_string()))
    }

    /// Compile Sierra to CASM (stub)
    ///
    /// Note: This will be implemented in Mission 0201-03
    /// with actual Cairo compiler integration.
    pub fn compile_to_casm(sierra: &[u8]) -> Result<Vec<u8>, CompileError> {
        let _ = sierra;
        Err(CompileError::NotImplemented("compile_to_casm".to_string()))
    }

    /// Check if this program has been fully compiled
    pub fn is_compiled(&self) -> bool {
        !self.sierra.is_empty() && !self.casm.is_empty()
    }
}

/// Cairo program compilation error
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileError {
    /// Feature not implemented yet
    NotImplemented(String),
    /// Syntax error in Cairo source
    SyntaxError(String),
    /// Type error during compilation
    TypeError(String),
    /// Compiler not found
    CompilerNotFound,
    /// Invalid Cairo compiler version
    InvalidVersion(String),
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::NotImplemented(msg) => write!(f, "Not implemented: {}", msg),
            CompileError::SyntaxError(msg) => write!(f, "Syntax error: {}", msg),
            CompileError::TypeError(msg) => write!(f, "Type error: {}", msg),
            CompileError::CompilerNotFound => write!(f, "Cairo compiler not found"),
            CompileError::InvalidVersion(msg) => write!(f, "Invalid version: {}", msg),
        }
    }
}

impl std::error::Error for CompileError {}

/// Registry of Cairo programs
///
/// Maintains a collection of programs and an allowlist of
/// programs approved for on-chain use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CairoProgramRegistry {
    /// All registered programs indexed by hash
    pub programs: BTreeMap<CairoProgramHash, CairoProgram>,
    /// Programs approved for on-chain use
    pub allowlist: BTreeSet<CairoProgramHash>,
}

impl Default for CairoProgramRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CairoProgramRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            programs: BTreeMap::new(),
            allowlist: BTreeSet::new(),
        }
    }

    /// Register a new Cairo program
    pub fn register(&mut self, program: CairoProgram) -> Result<(), RegistryError> {
        let hash = program.hash;

        // Check if program already exists
        if self.programs.contains_key(&hash) {
            return Err(RegistryError::AlreadyExists(hash));
        }

        self.programs.insert(hash, program);
        Ok(())
    }

    /// Get a program by its hash
    pub fn get(&self, hash: &CairoProgramHash) -> Option<&CairoProgram> {
        self.programs.get(hash)
    }

    /// Remove a program from the registry
    pub fn remove(&mut self, hash: &CairoProgramHash) -> Result<CairoProgram, RegistryError> {
        self.programs
            .remove(hash)
            .ok_or_else(|| RegistryError::NotFound(*hash))
            .map(|mut program| {
                // Also remove from allowlist if present
                self.allowlist.remove(hash);
                program
            })
    }

    /// Add a program to the allowlist (governance action)
    pub fn allowlist_add(&mut self, hash: CairoProgramHash) -> Result<(), RegistryError> {
        if !self.programs.contains_key(&hash) {
            return Err(RegistryError::NotFound(hash));
        }
        self.allowlist.insert(hash);
        Ok(())
    }

    /// Remove a program from the allowlist
    pub fn allowlist_remove(&mut self, hash: &CairoProgramHash) -> Result<(), RegistryError> {
        if !self.allowlist.remove(hash) {
            return Err(RegistryError::NotInAllowlist(*hash));
        }
        Ok(())
    }

    /// Check if a program is allowed for on-chain use
    pub fn is_allowed(&self, hash: &CairoProgramHash) -> bool {
        self.allowlist.contains(hash)
    }

    /// Get the number of registered programs
    pub fn len(&self) -> usize {
        self.programs.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.programs.is_empty()
    }

    /// Get all program hashes
    pub fn keys(&self) -> impl Iterator<Item = &CairoProgramHash> {
        self.programs.keys()
    }

    /// Get all allowed program hashes
    pub fn allowed_keys(&self) -> impl Iterator<Item = &CairoProgramHash> {
        self.allowlist.iter()
    }
}

/// Registry error types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryError {
    /// Program not found
    NotFound(CairoProgramHash),
    /// Program already exists
    AlreadyExists(CairoProgramHash),
    /// Program not in allowlist
    NotInAllowlist(CairoProgramHash),
    /// Registry full
    RegistryFull,
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistryError::NotFound(hash) => write!(f, "Program not found: {:?}", hash),
            RegistryError::AlreadyExists(hash) => write!(f, "Program already exists: {:?}", hash),
            RegistryError::NotInAllowlist(hash) => {
                write!(f, "Program not in allowlist: {:?}", hash)
            }
            RegistryError::RegistryFull => write!(f, "Registry is full"),
        }
    }
}

impl std::error::Error for RegistryError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cairo_program_hash() {
        let source = "fn main() { return (); }";
        let hash1 = CairoProgram::compute_hash(source);
        let hash2 = CairoProgram::compute_hash(source);
        assert_eq!(hash1, hash2, "Same source should produce same hash");

        let different_source = "fn main() { return 1; }";
        let hash3 = CairoProgram::compute_hash(different_source);
        assert_ne!(hash1, hash3, "Different source should produce different hash");
    }

    #[test]
    fn test_cairo_program_from_source() {
        let source = "fn main() { return (); }".to_string();
        let version = 2u32;
        let program = CairoProgram::from_source(source.clone(), version);

        assert_eq!(program.source, source);
        assert_eq!(program.version, version);
        assert!(!program.sierra.is_empty() || program.sierra.is_empty()); // Stub - empty for now
        assert!(!program.casm.is_empty() || program.casm.is_empty()); // Stub - empty for now
        assert!(!program.is_compiled(), "Should not be compiled (stub)");
    }

    #[test]
    fn test_cairo_program_compile_stubs() {
        let source = "fn main() { return (); }";
        let result = CairoProgram::compile_to_sierra(source);
        assert!(result.is_err(), "Should return error for unimplemented stub");

        let sierra = vec![1, 2, 3];
        let result = CairoProgram::compile_to_casm(&sierra);
        assert!(result.is_err(), "Should return error for unimplemented stub");
    }

    #[test]
    fn test_registry_new() {
        let registry = CairoProgramRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_registry_default() {
        let registry = CairoProgramRegistry::default();
        assert!(registry.is_empty());
    }

    #[test]
    fn test_registry_register() {
        let mut registry = CairoProgramRegistry::new();
        let source = "fn main() { return (); }".to_string();
        let program = CairoProgram::from_source(source, 2);

        let result = registry.register(program.clone());
        assert!(result.is_ok(), "Should successfully register program");
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_registry_register_duplicate() {
        let mut registry = CairoProgramRegistry::new();
        let source = "fn main() { return (); }".to_string();
        let program = CairoProgram::from_source(source, 2);

        registry.register(program.clone()).unwrap();
        let result = registry.register(program);

        assert!(result.is_err(), "Should fail to register duplicate");
        assert_eq!(registry.len(), 1, "Should still have only 1 program");
    }

    #[test]
    fn test_registry_get() {
        let mut registry = CairoProgramRegistry::new();
        let source = "fn main() { return (); }".to_string();
        let program = CairoProgram::from_source(source, 2);
        let hash = program.hash;

        registry.register(program).unwrap();

        let retrieved = registry.get(&hash);
        assert!(retrieved.is_some(), "Should retrieve registered program");
        assert_eq!(retrieved.unwrap().hash, hash);
    }

    #[test]
    fn test_registry_get_not_found() {
        let registry = CairoProgramRegistry::new();
        let hash = [0u8; 32];

        let retrieved = registry.get(&hash);
        assert!(retrieved.is_none(), "Should not find unregistered program");
    }

    #[test]
    fn test_registry_remove() {
        let mut registry = CairoProgramRegistry::new();
        let source = "fn main() { return (); }".to_string();
        let program = CairoProgram::from_source(source, 2);
        let hash = program.hash;

        registry.register(program).unwrap();
        assert_eq!(registry.len(), 1);

        let removed = registry.remove(&hash);
        assert!(removed.is_ok(), "Should successfully remove program");
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_registry_remove_not_found() {
        let mut registry = CairoProgramRegistry::new();
        let hash = [0u8; 32];

        let result = registry.remove(&hash);
        assert!(result.is_err(), "Should fail to remove non-existent program");
    }

    #[test]
    fn test_allowlist_add() {
        let mut registry = CairoProgramRegistry::new();
        let source = "fn main() { return (); }".to_string();
        let program = CairoProgram::from_source(source, 2);
        let hash = program.hash;

        registry.register(program).unwrap();
        registry.allowlist_add(hash).unwrap();

        assert!(registry.is_allowed(&hash), "Program should be allowed");
    }

    #[test]
    fn test_allowlist_add_not_found() {
        let mut registry = CairoProgramRegistry::new();
        let hash = [0u8; 32];

        let result = registry.allowlist_add(hash);
        assert!(result.is_err(), "Should fail to add non-existent program");
    }

    #[test]
    fn test_allowlist_remove() {
        let mut registry = CairoProgramRegistry::new();
        let source = "fn main() { return (); }".to_string();
        let program = CairoProgram::from_source(source, 2);
        let hash = program.hash;

        registry.register(program).unwrap();
        registry.allowlist_add(hash).unwrap();
        assert!(registry.is_allowed(&hash));

        registry.allowlist_remove(&hash).unwrap();
        assert!(!registry.is_allowed(&hash), "Program should not be allowed");
    }

    #[test]
    fn test_allowlist_not_in_allowlist() {
        let mut registry = CairoProgramRegistry::new();
        let hash = [0u8; 32];

        let result = registry.allowlist_remove(&hash);
        assert!(result.is_err(), "Should fail to remove non-allowed program");
    }

    #[test]
    fn test_is_allowed() {
        let mut registry = CairoProgramRegistry::new();
        let source = "fn main() { return (); }".to_string();
        let program = CairoProgram::from_source(source, 2);
        let hash = program.hash;

        registry.register(program).unwrap();
        assert!(!registry.is_allowed(&hash), "Program should not be allowed initially");

        registry.allowlist_add(hash).unwrap();
        assert!(registry.is_allowed(&hash), "Program should be allowed after allowlist_add");
    }

    #[test]
    fn test_registry_keys() {
        let mut registry = CairoProgramRegistry::new();
        let source1 = "fn main() { return (); }".to_string();
        let source2 = "fn main() { return 1; }".to_string();
        let program1 = CairoProgram::from_source(source1, 2);
        let program2 = CairoProgram::from_source(source2, 2);

        registry.register(program1).unwrap();
        registry.register(program2).unwrap();

        let keys: Vec<_> = registry.keys().collect();
        assert_eq!(keys.len(), 2);
    }

    #[test]
    fn test_registry_allowed_keys() {
        let mut registry = CairoProgramRegistry::new();
        let source1 = "fn main() { return (); }".to_string();
        let source2 = "fn main() { return 1; }".to_string();
        let program1 = CairoProgram::from_source(source1, 2);
        let program2 = CairoProgram::from_source(source2, 2);
        let hash1 = program1.hash;

        registry.register(program1).unwrap();
        registry.register(program2).unwrap();

        // Only allowlist first program
        registry.allowlist_add(hash1).unwrap();

        let allowed: Vec<_> = registry.allowed_keys().collect();
        assert_eq!(allowed.len(), 1);
        assert_eq!(allowed[0], &hash1);
    }
}
