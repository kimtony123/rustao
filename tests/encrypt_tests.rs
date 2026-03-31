#[cfg(test)]
mod tests {
    use rsa::{RsaPrivateKey, RsaPublicKey};
    use rustao::encrypt::{decrypt_payload, encrypt_payload};

    fn generate_test_keypair() -> (RsaPrivateKey, RsaPublicKey) {
        let mut rng = rand::thread_rng();
        let priv_key = RsaPrivateKey::new(&mut rng, 2048).unwrap();
        let pub_key = priv_key.to_public_key();
        (priv_key, pub_key)
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let (priv_key, pub_key) = generate_test_keypair();
        let plaintext = b"secret message";
        let (encrypted_data, encrypted_key, nonce) = encrypt_payload(plaintext, &pub_key).unwrap();
        let decrypted = decrypt_payload(&encrypted_data, &encrypted_key, &nonce, &priv_key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_empty() {
        let (_, pub_key) = generate_test_keypair();  // underscore for unused
        let plaintext = b"";
        let (_encrypted_data, _encrypted_key, _nonce) = encrypt_payload(plaintext, &pub_key).unwrap();
        // just ensure it doesn't panic
    }

    #[test]
    fn test_wrong_key() {
        let (priv_key, pub_key) = generate_test_keypair();
        let (other_priv, _) = generate_test_keypair();
        let plaintext = b"secret";
        let (encrypted_data, encrypted_key, nonce) = encrypt_payload(plaintext, &pub_key).unwrap();
        let result = decrypt_payload(&encrypted_data, &encrypted_key, &nonce, &other_priv);
        assert!(result.is_err());
    }
}