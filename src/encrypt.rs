use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit, Nonce};
use rand::RngCore;
use rsa::{Oaep, RsaPrivateKey, RsaPublicKey};
use sha2::Sha256;
use crate::error::{Error, Result};

const AES_KEY_SIZE: usize = 32;
const GCM_NONCE_SIZE: usize = 12;

/// Encrypt data using hybrid encryption: AES‑GCM + RSA‑OAEP.
pub fn encrypt_payload(
    plaintext: &[u8],
    rsa_pub: &RsaPublicKey,
) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
    // Generate random AES key and nonce
    let mut aes_key = vec![0u8; AES_KEY_SIZE];
    rand::thread_rng().fill_bytes(&mut aes_key);
    let mut nonce = vec![0u8; GCM_NONCE_SIZE];
    rand::thread_rng().fill_bytes(&mut nonce);

    // Encrypt with AES-GCM
    let cipher = Aes256Gcm::new_from_slice(&aes_key).map_err(|e| Error::Encryption(e.to_string()))?;
    let encrypted_data = cipher
        .encrypt(Nonce::from_slice(&nonce), plaintext)
        .map_err(|e| Error::Encryption(e.to_string()))?;

    // Encrypt AES key with RSA-OAEP (SHA‑256)
    let mut rng = rand::thread_rng();
    let encrypted_key = rsa_pub
        .encrypt(&mut rng, Oaep::new::<Sha256>(), &aes_key)
        .map_err(|e| Error::Encryption(e.to_string()))?;

    Ok((encrypted_data, encrypted_key, nonce))
}

/// Decrypt data using hybrid decryption: RSA‑OAEP to recover AES key, then AES‑GCM.
pub fn decrypt_payload(
    encrypted_data: &[u8],
    encrypted_key: &[u8],
    nonce: &[u8],
    rsa_priv: &RsaPrivateKey,
) -> Result<Vec<u8>> {
    // Decrypt AES key with RSA-OAEP
    let aes_key = rsa_priv
        .decrypt(Oaep::new::<Sha256>(), encrypted_key)
        .map_err(|e| Error::Encryption(e.to_string()))?;

    // Decrypt data with AES-GCM
    let cipher = Aes256Gcm::new_from_slice(&aes_key).map_err(|e| Error::Encryption(e.to_string()))?;
    let plaintext = cipher
        .decrypt(Nonce::from_slice(nonce), encrypted_data)
        .map_err(|e| Error::Encryption(e.to_string()))?;

    Ok(plaintext)
}