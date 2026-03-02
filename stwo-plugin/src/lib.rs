// stwo-plugin/src/lib.rs
// STWO verification plugin - exports C-compatible functions

mod verify;

use std::os::raw::c_char;
use std::ffi::CString;

/// Result of a verification operation
#[repr(C)]
pub struct StarkVerifyResult {
    pub success: bool,
    pub error_message: *const c_char,
}

/// Plugin version string
pub const PLUGIN_VERSION: &str = "1.0.0";

/// Get the plugin version
///
/// Returns a pointer to a null-terminated string containing the version.
#[no_mangle]
pub extern "C" fn stark_plugin_version() -> *const c_char {
    CString::new(PLUGIN_VERSION).unwrap().into_raw()
}

/// Verify a STARK proof using STWO
///
/// Takes a pointer to proof bytes and verifies it using STWO.
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
    if proof_bytes.is_null() {
        return StarkVerifyResult {
            success: false,
            error_message: CString::new("Null proof pointer").unwrap().into_raw(),
        };
    }

    let proof_slice = std::slice::from_raw_parts(proof_bytes, proof_len);

    // Verify the proof
    match verify::verify_proof_internal(proof_slice) {
        Ok(true) => StarkVerifyResult {
            success: true,
            error_message: std::ptr::null(),
        },
        Ok(false) => StarkVerifyResult {
            success: false,
            error_message: CString::new("Proof verification failed").unwrap().into_raw(),
        },
        Err(e) => StarkVerifyResult {
            success: false,
            error_message: CString::new(e.to_string()).unwrap().into_raw(),
        },
    }
}

/// Verify a STARK proof with metadata
///
/// Takes additional metadata for verification:
/// - program_hash: expected program hash (32 bytes)
/// - outputs: expected outputs from the proof
///
/// # Safety
/// - All pointers must be valid
/// - All length parameters must match actual data
#[no_mangle]
pub unsafe extern "C" fn stark_verify_proof_with_metadata(
    proof_bytes: *const u8,
    proof_len: usize,
    _program_hash: *const u8,
    _outputs: *const u8,
    _outputs_len: usize,
) -> StarkVerifyResult {
    // For now, just do basic verification without metadata check
    // The metadata checking can be added based on requirements
    stark_verify_proof(proof_bytes, proof_len)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;

    #[test]
    fn test_plugin_version() {
        let version = unsafe { CStr::from_ptr(stark_plugin_version()) }
            .to_string_lossy()
            .into_owned();
        assert_eq!(version, "1.0.0");
    }
}
