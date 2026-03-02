# Mission: Compressed Proof Verification

## Status
Open

## RFC
RFC-0202: Compressed Proof Format for Batch Verification
RFC-0205: STWO Verification Plugin Architecture

## Acceptance Criteria
- [ ] Create stwo-plugin crate with C-compatible verification
- [ ] Implement `CompressedProof::verify()` using plugin
- [ ] Add plugin discovery (env var + default path)
- [ ] Add proper error handling (PluginNotFound, PluginError)
- [ ] Verify STARK proof via STWO when plugin available
- [ ] Check program hash against registry
- [ ] Add tests for valid proof verification
- [ ] Add tests for invalid proof rejection
- [ ] Add tests for missing plugin scenario
- [ ] Benchmark verification time (target: <100ms)

## Dependencies
- Mission 0201-05 (Prover Interface)
- Mission 0202-01 (Compressed Proof Types)
- Mission 0202-02 (Proof Generation)

## Enables
- RFC-0202 completion
- RFC-0205 completion

## Implementation Notes

### Files to Create

**stwo-plugin/ (new crate outside workspace)**
- `stwo-plugin/Cargo.toml` - cdylib + rlib crate types
- `stwo-plugin/rust-toolchain.toml` - nightly-2025-06-23
- `stwo-plugin/src/lib.rs` - C-compatible exports
- `stwo-plugin/src/verify.rs` - STWO verification wrapper

### Files to Modify

- `src/zk/compressed.rs` - Add verify() using plugin
- `src/zk/plugin.rs` - NEW: Plugin loading + error types
- `src/zk/mod.rs` - Export plugin module

### Implementation

```rust
// src/zk/plugin.rs - NEW FILE

use libloading::{Library, Symbol};
use std::path::Path;

#[repr(C)]
pub struct StarkVerifyResult {
    pub success: bool,
    pub error_message: *const std::os::raw::c_char,
}

pub struct STWOPlugin {
    lib: Library,
}

impl STWOPlugin {
    pub fn load(path: &Path) -> Result<Self, PluginError> { ... }
    pub fn verify(&self, proof: &[u8]) -> Result<bool, PluginError> { ... }
}

pub fn load_plugin() -> Result<STWOPlugin, PluginError> { ... }
```

```rust
// src/zk/compressed.rs - UPDATE

impl CompressedProof {
    pub fn verify(&self) -> Result<bool, CompressedProofError> {
        self.validate()?;
        let plugin = crate::zk::plugin::load_plugin()
            .map_err(CompressedProofError::PluginNotFound)?;
        plugin.verify(&self.stark_proof.proof)
            .map_err(CompressedProofError::PluginError)
    }
}
```

### Plugin Interface

```rust
// stwo-plugin/src/lib.rs

#[no_mangle]
pub unsafe extern "C" fn stark_verify_proof(
    proof_bytes: *const u8,
    proof_len: usize,
) -> StarkVerifyResult { ... }

#[no_mangle]
pub extern "C" fn stark_plugin_version() -> *const c_char { ... }
```

### Discovery

1. Environment variable: `STOOLAP_STWO_PLUGIN`
2. Default path: `../stwo-plugin/target/release/libstwo_plugin.so`
3. Error: Clear message with setup instructions

### Tests

```rust
#[test]
fn test_verify_with_plugin() {
    // Requires plugin to be built
    let proof = load_test_proof();
    let result = proof.verify();
    assert!(result.is_ok());
}

#[test]
fn test_verify_missing_plugin() {
    // Without plugin, should return PluginNotFound
    let proof = load_test_proof();
    match proof.verify() {
        Err(CompressedProofError::PluginNotFound) => {}
        _ => panic!("Expected PluginNotFound"),
    }
}

#[test]
fn test_verify_invalid_proof() {
    // Tamper with proof, should fail
}
```

## Claimant
Open

## Pull Request
TBD

## Commits
TBD

## Completion Date
TBD
