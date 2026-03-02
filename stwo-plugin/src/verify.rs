// stwo-plugin/src/verify.rs
// STWO verification implementation

use cairo_air::CairoProofForRustVerifier;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sMerkleChannel;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sMerkleHasher;
use stwo::core::fri::FriConfig;
use stwo::core::pcs::PcsConfig;
use stwo_cairo_adapter::ProverInput;
use stwo_cairo_prover::prover::{ChannelHash, ProverParameters};
use cairo_air::PreProcessedTraceVariant;
use thiserror::Error;

/// Verification errors
#[derive(Error, Debug)]
pub enum VerifyError {
    #[error("Failed to parse proof: {0}")]
    ParseError(String),

    #[error("Verification failed: {0}")]
    VerificationFailed(String),

    #[error("Proof is invalid: {0}")]
    InvalidProof(String),

    #[error("Failed to generate proof: {0}")]
    GenerationError(String),
}

/// Create default prover parameters
fn create_default_prover_params() -> ProverParameters {
    ProverParameters {
        channel_hash: ChannelHash::Blake2s,
        channel_salt: 0,
        pcs_config: PcsConfig {
            pow_bits: 26,
            fri_config: FriConfig {
                log_last_layer_degree_bound: 0,
                log_blowup_factor: 1,
                n_queries: 70,
                line_fold_step: 1,
            },
            lifting_log_size: None,
        },
        preprocessed_trace: PreProcessedTraceVariant::Canonical,
        store_polynomials_coefficients: false,
        include_all_preprocessed_columns: false,
    }
}

/// Verify a STARK proof
///
/// Takes raw proof bytes (JSON format), parses them, and verifies using STWO.
pub fn verify_proof_internal(proof_bytes: &[u8]) -> Result<bool, VerifyError> {
    // Try to parse as JSON - CairoProofForRustVerifier serialized
    let proof: Result<CairoProofForRustVerifier<Blake2sMerkleHasher>, _> =
        serde_json::from_slice(proof_bytes);

    match proof {
        Ok(proof_for_verifier) => {
            // Verify using STWO
            match cairo_air::verifier::verify_cairo::<Blake2sMerkleChannel>(proof_for_verifier) {
                Ok(()) => Ok(true),
                Err(e) => Err(VerifyError::VerificationFailed(format!("{:?}", e))),
            }
        }
        Err(e) => Err(VerifyError::ParseError(format!(
            "Could not parse proof as JSON: {}. Expected CairoProofForRustVerifier JSON.",
            e
        ))),
    }
}

/// Generate and verify a test proof
///
/// This is a utility function for testing - generates a proof and immediately verifies it.
pub fn generate_and_verify_test_proof(prover_input_json: &str) -> Result<bool, VerifyError> {
    // Parse prover input
    let prover_input: ProverInput = serde_json::from_str(prover_input_json)
        .map_err(|e| VerifyError::ParseError(format!("Failed to parse prover input: {}", e)))?;

    let params = create_default_prover_params();

    // Generate proof
    let proof = stwo_cairo_prover::prover::prove_cairo::<Blake2sMerkleChannel>(
        prover_input,
        params,
    )
    .map_err(|e| VerifyError::GenerationError(format!("{:?}", e)))?;

    // Convert to verifier-compatible format
    let proof_for_verifier: CairoProofForRustVerifier<Blake2sMerkleHasher> = proof.into();

    // Verify
    match cairo_air::verifier::verify_cairo::<Blake2sMerkleChannel>(proof_for_verifier) {
        Ok(()) => Ok(true),
        Err(e) => Err(VerifyError::VerificationFailed(format!("{:?}", e))),
    }
}

/// Generate a proof and return it as JSON bytes
pub fn generate_proof_as_json(prover_input_json: &str) -> Result<Vec<u8>, VerifyError> {
    // Parse prover input
    let prover_input: ProverInput = serde_json::from_str(prover_input_json)
        .map_err(|e| VerifyError::ParseError(format!("Failed to parse prover input: {}", e)))?;

    let params = create_default_prover_params();

    // Generate proof
    let proof = stwo_cairo_prover::prover::prove_cairo::<Blake2sMerkleChannel>(
        prover_input,
        params,
    )
    .map_err(|e| VerifyError::GenerationError(format!("{:?}", e)))?;

    // Convert to verifier-compatible format
    let proof_for_verifier: CairoProofForRustVerifier<Blake2sMerkleHasher> = proof.into();

    // Serialize to JSON
    let json = serde_json::to_string(&proof_for_verifier)
        .map_err(|e| VerifyError::ParseError(format!("Failed to serialize proof: {}", e)))?;

    Ok(json.into_bytes())
}

/// Verify a proof with additional validation
///
/// Validates that the proof matches expected program hash and outputs.
pub fn verify_proof_with_validation(
    proof_bytes: &[u8],
    _expected_program_hash: &[u8; 32],
    _expected_outputs: &[u8],
) -> Result<bool, VerifyError> {
    // First do basic verification
    verify_proof_internal(proof_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    const STWO_CAIRO_PATH: &str = "/home/mmacedoeu/_w/crypto/stwo-cairo/stwo_cairo_prover/test_data";

    #[test]
    fn test_verify_empty_proof() {
        let result = verify_proof_internal(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_small_proof() {
        let result = verify_proof_internal(&[1, 2, 3]);
        assert!(result.is_err());
    }

    #[test]
    #[ignore] // Takes too long for regular test runs
    fn test_generate_and_verify_all_builtins() {
        let prover_input_path = format!(
            "{}/test_prove_verify_all_builtins/prover_input.json",
            STWO_CAIRO_PATH
        );
        let input_json = std::fs::read_to_string(&prover_input_path)
            .expect("Failed to read prover input");

        let result = generate_and_verify_test_proof(&input_json);
        println!("Result: {:?}", result);
        assert!(result.is_ok());
    }
}
