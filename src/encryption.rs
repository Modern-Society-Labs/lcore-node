use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use anyhow::Result;

pub struct EncryptedPayload {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

/// A placeholder for the dual encryption logic.
/// In this MVP, we just perform one layer of AES-256-GCM encryption.
pub fn encrypt_stage1(data: &[u8]) -> Result<EncryptedPayload> {
    let key = Aes256Gcm::generate_key(OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(b"unique nonce"); // 96-bits; unique per message
    
    let ciphertext = cipher.encrypt(nonce, data)
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

    Ok(EncryptedPayload {
        ciphertext,
        nonce: nonce.to_vec(),
    })
} 