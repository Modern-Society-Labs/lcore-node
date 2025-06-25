// lcore-node/src/device_auth.rs
//
// Phase 3: Device Authentication using DID/JOSE

use crate::error::LCoreError;
use josekit::jwk::Jwk;
use josekit::jws::{EdDSA, JwsVerifier};
use base64::{Engine as _, engine::general_purpose};

/// Verifies a JWS signature against a public key.
/// In a real system, the JWK would be resolved from a DID document.
pub fn verify_device_signature(jws: &str, payload: &[u8], public_jwk_json: &str) -> Result<(), LCoreError> {
    // 1. Parse the public key from the provided JWK JSON.
    let jwk = Jwk::from_bytes(public_jwk_json.as_bytes())
        .map_err(|e| LCoreError::DeviceAuth(format!("Failed to parse JWK: {}", e)))?;

    // 2. Split the JWS into its components
    let parts: Vec<&str> = jws.split('.').collect();
    if parts.len() != 3 {
        return Err(LCoreError::DeviceAuth("Invalid JWS format".to_string()));
    }

    // 3. Decode the header and payload
    let _header_bytes = general_purpose::URL_SAFE_NO_PAD.decode(parts[0])
        .map_err(|e| LCoreError::DeviceAuth(format!("Failed to decode JWS header: {}", e)))?;
    
    let payload_bytes = general_purpose::URL_SAFE_NO_PAD.decode(parts[1])
        .map_err(|e| LCoreError::DeviceAuth(format!("Failed to decode JWS payload: {}", e)))?;
    
    let signature_bytes = general_purpose::URL_SAFE_NO_PAD.decode(parts[2])
        .map_err(|e| LCoreError::DeviceAuth(format!("Failed to decode JWS signature: {}", e)))?;

    // 4. Verify the payload matches what we expect
    if payload_bytes != payload {
        return Err(LCoreError::DeviceAuth("Payload does not match signature".to_string()));
    }

    // 5. Create the message to verify (header.payload)
    let message = format!("{}.{}", parts[0], parts[1]);
    
    // 6. Create a verifier for Ed25519 algorithm (most common for DID)
    let verifier = EdDSA.verifier_from_jwk(&jwk)
        .map_err(|e| LCoreError::DeviceAuth(format!("Failed to create verifier: {}", e)))?;

    // 7. Verify the signature
    verifier.verify(message.as_bytes(), &signature_bytes)
        .map_err(|e| LCoreError::DeviceAuth(format!("Invalid JWS signature: {}", e)))?;

    Ok(())
}

