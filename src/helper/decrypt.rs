// src/services/crypto.rs

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use hex;
use std::env;

/// Decrypts AES256-GCM encrypted data.
///
/// Takes the encrypted bytes, the hexadecimal representation of the AES key,
/// and the hexadecimal representation of the GCM Nonce (IV).
///
/// Returns `Ok(Vec<u8>)` with the decrypted bytes on success,
/// or `Err(String)` with an error message on failure.
pub fn decrypt_aes_256_gcm(
    encrypted_data: &[u8],
    aes_key_hex: &str,
    aes_nonce_hex: &str,
) -> Result<Vec<u8>, String> {
    let key_bytes =
        hex::decode(aes_key_hex).map_err(|e| format!("Invalid AES_KEY_HEX format: {}", e))?;
    let nonce_bytes =
        hex::decode(aes_nonce_hex).map_err(|e| format!("Invalid AES_NONCE_HEX format: {}", e))?;

    if key_bytes.len() != 32 {
        // 32 bytes for AES256
        return Err("AES_KEY_HEX must decode to 32 bytes.".to_string());
    }
    if nonce_bytes.len() != 12 {
        // 12 bytes for GCM Nonce
        return Err("AES_NONCE_HEX must decode to 12 bytes.".to_string());
    }

    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&nonce_bytes); // 96-bits; unique per encryption

    cipher.decrypt(nonce, encrypted_data).map_err(|e| {
        format!(
            "Decryption failed: {}. Ensure key, IV, and data are correct.",
            e
        )
    })
}
