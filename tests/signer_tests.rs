#[cfg(test)]
mod tests {
    use rustao::ARSigner;
    use std::path::PathBuf;

    fn test_wallet_path() -> PathBuf {
        PathBuf::from("testKey.json")
    }

    #[test]
    fn test_load_wallet() {
        let signer = ARSigner::from_file(test_wallet_path()).expect("failed to load wallet");
        assert!(!signer.address().is_empty());
        assert!(!signer.public_key().is_empty());
    }

    #[test]
    fn test_sign() {
        let signer = ARSigner::from_file(test_wallet_path()).unwrap();
        let data = b"test data";
        let signature = signer.sign(data).unwrap();
        assert!(!signature.is_empty());
    }

    #[test]
    fn test_public_key() {
        let signer = ARSigner::from_file(test_wallet_path()).unwrap();
        let pk = signer.public_key();
        assert!(!pk.is_empty());
    }

    #[test]
    fn test_address() {
        let signer = ARSigner::from_file(test_wallet_path()).unwrap();
        let addr = signer.address();
        assert!(!addr.is_empty());
        // Address is base58, so it should contain only alphanumeric characters
        assert!(addr.chars().all(|c| c.is_ascii_alphanumeric()));
    }
}