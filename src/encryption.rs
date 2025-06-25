// lcore-node/src/encryption.rs
//
// Phase 3: Dual Encryption System Implementation

use aes_gcm::{Aes256Gcm, Nonce};
use aes_gcm::aead::{Aead, KeyInit};
use chacha20poly1305::XChaCha20Poly1305;
use crate::error::LCoreError;
use sha2::{Sha256, Digest};
use byteorder::{BigEndian, WriteBytesExt};

/// Stage 1 Encryption using AES-256-GCM
pub struct Stage1Encryption {
    key: [u8; 32],
}

impl Stage1Encryption {
    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, LCoreError> {
        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|_| LCoreError::Encryption("Invalid AES key length".to_string()))?;
        
        // NOTE: In a real implementation, the nonce should be deterministic and unique per message.
        // For this implementation, we will use a fixed nonce for simplicity, but this is NOT secure for production.
        // A better approach would be to derive it from a counter or the input data itself.
        let nonce = Nonce::from_slice(b"unique nonce"); // 96-bits
        
        cipher.encrypt(nonce, plaintext)
            .map_err(|_| LCoreError::Encryption("AES encryption failed".to_string()))
    }

    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, LCoreError> {
        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|_| LCoreError::Encryption("Invalid AES key length".to_string()))?;
        let nonce = Nonce::from_slice(b"unique nonce"); // Nonce must match encryption
        
        cipher.decrypt(nonce, ciphertext)
            .map_err(|_| LCoreError::Encryption("AES decryption failed".to_string()))
    }

    /// Deterministic encryption with caller-supplied nonce (12 bytes)
    pub fn encrypt_with_nonce(&self, plaintext: &[u8], nonce_bytes: &[u8; 12]) -> Result<Vec<u8>, LCoreError> {
        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|_| LCoreError::Encryption("Invalid AES key length".to_string()))?;
        let nonce = Nonce::from_slice(nonce_bytes);
        cipher.encrypt(nonce, plaintext)
            .map_err(|_| LCoreError::Encryption("AES encryption failed".to_string()))
    }

    pub fn decrypt_with_nonce(&self, ciphertext: &[u8], nonce_bytes: &[u8; 12]) -> Result<Vec<u8>, LCoreError> {
        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|_| LCoreError::Encryption("Invalid AES key length".to_string()))?;
        let nonce = Nonce::from_slice(nonce_bytes);
        cipher.decrypt(nonce, ciphertext)
            .map_err(|_| LCoreError::Encryption("AES decryption failed".to_string()))
    }

    pub fn derive_key_from_did(did: &str) -> Result<[u8; 32], LCoreError> {
        // Use SHA-256 to create a deterministic 32-byte key from the DID string.
        let mut hasher = Sha256::new();
        hasher.update(did.as_bytes());
        let result = hasher.finalize();
        Ok(result.into())
    }
}

/// Stage 2 Encryption using XChaCha20-Poly1305
pub struct Stage2Encryption {
    key: [u8; 32],
}

impl Stage2Encryption {
    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    pub fn encrypt(&self, stage1_ciphertext: &[u8]) -> Result<Vec<u8>, LCoreError> {
        let cipher = XChaCha20Poly1305::new_from_slice(&self.key)
            .map_err(|_| LCoreError::Encryption("Invalid ChaCha key length".to_string()))?;
        
        // NOTE: As with AES, the nonce should be deterministic and unique.
        // For this implementation, we will use a fixed nonce for simplicity.
        let nonce = [0u8; 24]; // XChaCha20 uses a 24-byte nonce

        cipher
            .encrypt(&nonce.into(), stage1_ciphertext)
            .map_err(|_| LCoreError::Encryption("ChaCha encryption failed".to_string()))
    }

    pub fn decrypt(&self, stage2_ciphertext: &[u8]) -> Result<Vec<u8>, LCoreError> {
        let cipher = XChaCha20Poly1305::new_from_slice(&self.key)
            .map_err(|_| LCoreError::Encryption("Invalid ChaCha key length".to_string()))?;
        let nonce = [0u8; 24]; // Nonce must match encryption

        cipher
            .decrypt(&nonce.into(), stage2_ciphertext)
            .map_err(|_| LCoreError::Encryption("ChaCha decryption failed".to_string()))
    }

    pub fn encrypt_with_nonce(&self, stage1_ciphertext: &[u8], nonce_bytes: &[u8; 24]) -> Result<Vec<u8>, LCoreError> {
        let cipher = XChaCha20Poly1305::new_from_slice(&self.key)
            .map_err(|_| LCoreError::Encryption("Invalid ChaCha key length".to_string()))?;
        let nonce = chacha20poly1305::XNonce::from_slice(nonce_bytes);
        cipher.encrypt(nonce, stage1_ciphertext)
            .map_err(|_| LCoreError::Encryption("ChaCha encryption failed".to_string()))
    }

    pub fn decrypt_with_nonce(&self, stage2_ciphertext: &[u8], nonce_bytes: &[u8; 24]) -> Result<Vec<u8>, LCoreError> {
        let cipher = XChaCha20Poly1305::new_from_slice(&self.key)
            .map_err(|_| LCoreError::Encryption("Invalid ChaCha key length".to_string()))?;
        let nonce = chacha20poly1305::XNonce::from_slice(nonce_bytes);
        cipher.decrypt(nonce, stage2_ciphertext)
            .map_err(|_| LCoreError::Encryption("ChaCha decryption failed".to_string()))
    }

    pub fn derive_key_from_context(context: &str) -> Result<[u8; 32], LCoreError> {
        // Use SHA-256 to create a deterministic 32-byte key from a context string.
        let mut hasher = Sha256::new();
        hasher.update(context.as_bytes());
        let result = hasher.finalize();
        Ok(result.into())
    }
}

/// Derive 96-bit AES-GCM nonce per systemPatterns §10
pub fn derive_stage1_nonce(device_id: &str, counter: u64) -> [u8; 12] {
    let mut hasher = Sha256::new();
    hasher.update(device_id.as_bytes());
    let mut counter_bytes = [0u8; 8];
    (&mut counter_bytes[..]).write_u64::<BigEndian>(counter).unwrap();
    hasher.update(counter_bytes);
    let digest = hasher.finalize();
    let mut out = [0u8; 12];
    out.copy_from_slice(&digest[0..12]);
    out
}

/// Derive 192-bit XChaCha20-Poly1305 nonce per systemPatterns §10
pub fn derive_stage2_nonce(device_id: &str, counter: u64) -> [u8; 24] {
    let mut hasher = Sha256::new();
    hasher.update(device_id.as_bytes());
    let mut counter_bytes = [0u8; 8];
    (&mut counter_bytes[..]).write_u64::<BigEndian>(counter).unwrap();
    hasher.update(counter_bytes);
    hasher.update(b"stage2");
    let digest = hasher.finalize();
    let mut out = [0u8; 24];
    out.copy_from_slice(&digest[0..24]);
    out
} 