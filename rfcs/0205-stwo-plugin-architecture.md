# RFC-0205: STWO Verification Plugin Architecture

## Status
Draft

## Summary

Define a plugin architecture for STWO verification that allows the root crate to compile on stable Rust while enabling full STARK proof verification when a compiled plugin is available.

## Motivation

### The Problem

The STWO library (`stwo = "2.1"`) requires nightly Rust due to unstable features:
- `array_chunks`
- `portable_simd`
- `slice_ptr_get`

This creates a challenge:
1. **Root crate** must compile on stable Rust for broad compatibility
2. **STWO verification** requires nightly Rust
3. **zk feature** was designed to gate STWO dependencies, but still fails on stable

### Current State

- `stoolap` with `zk` feature → requires nightly Rust
- `stwo-bench` crate → works with nightly, only for benchmarks
- No way to verify proofs in the main crate on stable Rust

### The Solution

Create a separate `stwo-plugin` crate that:
1. Compiles to a `.so` dynamic library with nightly Rust
2. Exports C-compatible verification functions
3. Loads dynamically at runtime from the main crate
4. Provides clear error message when plugin is not available

## Specification

### Architecture Overview

```
stoolap_chain/           # Root crate (stable Rust)
├── src/zk/
│   ├── plugin.rs        # Runtime loading + trait definitions
│   └── compressed.rs    # Uses plugin for verification
└── Cargo.toml           # No stwo dependency

../stwo-plugin/          # Plugin crate (nightly Rust)
├── src/
│   ├── lib.rs           # C-compatible exports
│   └── verify.rs        # STWO verification wrapper
├── Cargo.toml           # stwo dependencies + cdylib
└── rust-toolchain.toml  # nightly-2025-06-23
```

### Plugin Interface

The plugin exposes C-compatible functions via `#[no_mangle]` and `extern "C"`:

```rust
// stwo-plugin/src/lib.rs

use std::os::raw::c_char;

/// Result of a verification operation
#[repr(C)]
pub struct StarkVerifyResult {
    pub success: bool,
    pub error_message: *const c_char,
}

/// Verify a STARK proof using STWO
///
/// # Safety
/// - proof_bytes must point to valid memory
/// - proof_len must be the actual byte length
#[no_mangle]
pub unsafe extern "C" fn stark_verify_proof(
    proof_bytes: *const u8,
    proof_len: usize,
) -> StarkVerifyResult {
    // Convert raw pointer to slice
    let proof_slice = std::slice::from_raw_parts(proof_bytes, proof_len);

    // Parse and verify using STWO
    match verify_proof_internal(proof_slice) {
        Ok(true) => StarkVerifyResult {
            success: true,
            error_message: std::ptr::null(),
        },
        Ok(false) => StarkVerifyResult {
            success: false,
            error_message: c_string!("Proof verification failed"),
        },
        Err(e) => StarkVerifyResult {
            success: false,
            error_message: c_string!(e.to_string()),
        },
    }
}

/// Get the plugin version
#[no_mangle]
pub extern "C" fn stark_plugin_version() -> *const c_char {
    c_string!("1.0.0")
}
```

### Root Crate Loading

```rust
// src/zk/plugin.rs

use libloading::{Library, Symbol};
use std::path::Path;

/// Errors when plugin is not available
#[derive(Debug, Clone)]
pub enum PluginError {
    /// Plugin file not found
    NotFound,
    /// Failed to load the plugin library
    LoadFailed(String),
    /// Required symbol not found in plugin
    SymbolNotFound,
    /// Verification failed
    VerificationFailed(String),
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginError::NotFound => write!(
                f,
                "STWO plugin not found. Set STOOLAP_STWO_PLUGIN environment variable \
                or build the stwo-plugin crate. See: https://github.com/stoolap/stwo-plugin"
            ),
            PluginError::LoadFailed(msg) => write!(f, "Failed to load STWO plugin: {}", msg),
            PluginError::SymbolNotFound => write!(f, "STWO plugin missing required symbols"),
            PluginError::VerificationFailed(msg) => write!(f, "STWO verification failed: {}", msg),
        }
    }
}

impl std::error::Error for PluginError {}

/// STWO Plugin wrapper
pub struct STWOPlugin {
    lib: Library,
}

impl STWOPlugin {
    /// Load the plugin from a .so file
    pub fn load(path: &Path) -> Result<Self, PluginError> {
        let lib = unsafe { Library::new(path) }
            .map_err(|e| PluginError::LoadFailed(e.to_string()))?;
        Ok(Self { lib })
    }

    /// Verify a proof using the loaded plugin
    pub fn verify(&self, proof: &[u8]) -> Result<bool, PluginError> {
        type VerifyFn = unsafe extern "C" fn(*const u8, usize) -> StarkVerifyResult;

        let func: Symbol<VerifyFn> = self.lib.get(b"stark_verify_proof")
            .map_err(|_| PluginError::SymbolNotFound)?;

        let result = unsafe { func(proof.as_ptr(), proof.len()) };

        if result.success {
            Ok(true)
        } else {
            let msg = unsafe { std::ffi::CStr::from_ptr(result.error_message) }
                .to_string_lossy()
                .into_owned();
            Err(PluginError::VerificationFailed(msg))
        }
    }

    /// Get plugin version
    pub fn version(&self) -> Result<String, PluginError> {
        type VersionFn = extern "C" fn() -> *const c_char;

        let func: Symbol<VersionFn> = self.lib.get(b"stark_plugin_version")
            .map_err(|_| PluginError::SymbolNotFound)?;

        let version_cstr = unsafe { std::ffi::CStr::from_ptr(func()) };
        Ok(version_cstr.to_string_lossy().into_owned())
    }
}

/// Try to load the STWO plugin
///
/// Search order:
/// 1. Environment variable `STOOLAP_STWO_PLUGIN`
/// 2. Default path `../stwo-plugin/target/release/libstwo_plugin.so`
pub fn load_plugin() -> Result<STWOPlugin, PluginError> {
    // 1. Check environment variable
    if let Ok(path) = std::env::var("STOOLAP_STWO_PLUGIN") {
        return STWOPlugin::load(Path::new(&path));
    }

    // 2. Check default path
    let default_path = Path::new("..")
        .join("stwo-plugin")
        .join("target")
        .join("release")
        .join("libstwo_plugin.so");

    if default_path.exists() {
        return STWOPlugin::load(&default_path);
    }

    // 3. Not found
    Err(PluginError::NotFound)
}
```

### Integration with CompressedProof

```rust
// src/zk/compressed.rs

impl CompressedProof {
    /// Verify the compressed proof using STWO plugin
    ///
    /// Requires the STWO plugin to be available.
    /// Set `STOOLAP_STWO_PLUGIN` environment variable or ensure
    /// `../stwo-plugin/target/release/libstwo_plugin.so` exists.
    ///
    /// # Errors
    ///
    /// Returns `CompressedProofError::PluginNotFound` if plugin is not available.
    pub fn verify(&self) -> Result<bool, CompressedProofError> {
        // 1. Validate proof structure
        self.validate()?;

        // 2. Load plugin (no fallback)
        let plugin = crate::zk::plugin::load_plugin()
            .map_err(CompressedProofError::PluginNotFound)?;

        // 3. Verify proof
        plugin.verify(&self.stark_proof.proof)
            .map_err(CompressedProofError::PluginError)
    }
}
```

### Error Handling

```rust
// Add to CompressedProofError enum
pub enum CompressedProofError {
    // ... existing variants

    /// Plugin was not found
    PluginNotFound,

    /// Plugin error occurred
    PluginError(PluginError),
}
```

### Discovery Mechanisms

| Method | Priority | Description |
|--------|----------|-------------|
| Environment variable | 1st | `STOOLAP_STWO_PLUGIN=/path/to/libstwo_plugin.so` |
| Default path | 2nd | `../stwo-plugin/target/release/libstwo_plugin.so` |
| Error | - | Clear message with setup instructions |

## Plugin Crate Structure

### Cargo.toml

```toml
[package]
name = "stwo-plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
cairo-air = { git = "https://github.com/starkware-libs/stwo-cairo.git", tag = "v1.1.0" }
stwo = "2.1"
serde = { version = "1.0", features = ["derive"] }
```

### rust-toolchain.toml

```toml
[toolchain]
channel = "nightly-2025-06-23"
components = ["rustfmt", "clippy"]
```

## Usage Examples

### Building the Plugin

```bash
# Clone and build the plugin
$ cd ../stwo-plugin
$ cargo build --release

# Output: target/release/libstwo_plugin.so
```

### Using in Application

```rust
use stoolap::zk::CompressedProof;

fn main() {
    // Load proof from file/database
    let proof = load_compressed_proof();

    match proof.verify() {
        Ok(true) => println!("Proof verified successfully!"),
        Ok(false) => println!("Proof is invalid!"),
        Err(CompressedProofError::PluginNotFound) => {
            eprintln!("STWO plugin not found.");
            eprintln!("Build it from: https://github.com/stoolap/stwo-plugin");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Verification error: {}", e);
            std::process::exit(1);
        }
    }
}
```

### Environment Variable

```bash
# Use custom path
$ export STOOLAP_STWO_PLUGIN=/custom/path/libstwo_plugin.so
$ ./my_app
```

## Rationale

### Why Dynamic Loading?

1. **Compiler independence** - Root crate stays on stable Rust
2. **Plugin version flexibility** - Can upgrade plugin without recompiling main crate
3. **Clear separation** - Nightly-only code isolated in plugin
4. **Graceful degradation** - Clear error when plugin unavailable

### Why Not Feature Gating?

The `zk` feature approach was attempted but:
- Still requires nightly Rust to compile
- Doesn't support cross-compilation
- All-or-nothing approach

### Why Not Git Submodule?

- More complex setup for users
- Tightly couples plugin development
- Harder to version independently

### Why libloading?

- Standard Rust crate for dynamic loading
- Works with any `.so` file
- No build-time dependency

## Implementation Path

### Phase 1: Create Plugin Crate

1. Create `../stwo-plugin/` directory
2. Add `Cargo.toml` with stwo dependencies
3. Add `rust-toolchain.toml`
4. Implement verification wrapper

### Phase 2: Update Root Crate

1. Add `libloading` dependency
2. Create `src/zk/plugin.rs`
3. Update `CompressedProof::verify()`
4. Add error variants

### Phase 3: Documentation

1. Update RFC-0202 verification section
2. Create setup documentation
3. Add examples

## Dependencies

- RFC-0201: STWO/Cairo Integration
- RFC-0202: Compressed Proof Format

## Related RFCs

- [RFC-0202: Compressed Proof Format](./0202-compressed-proofs.md)

## Open Questions

1. Should we support Windows (.dll)?
2. Should we cache the loaded plugin?
3. Should we support plugin versioning?
