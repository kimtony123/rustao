use rand::rngs::OsRng;
use rsa::pkcs1v15::{Signature, SigningKey};
use rsa::signature::{RandomizedSigner, SignatureEncoding, Verifier};
use rsa::traits::PublicKeyParts;
use rsa::BigUint;
use rsa::RsaPrivateKey;
use sha2::Sha256;

pub struct Driver;

pub struct PrivateKey(RsaPrivateKey);

impl PrivateKey {
    pub fn sign(&self, data: &[u8]) -> Vec<u8> {
        let mut rng = OsRng;
        let signing_key = SigningKey::<Sha256>::new_unprefixed(self.0.clone());
        let signature: Signature = signing_key.sign_with_rng(&mut rng, data);
        signature.to_bytes().to_vec()
    }

    pub fn verify(&self, data: &[u8], signature: &[u8]) -> bool {
        use rsa::pkcs1v15::VerifyingKey;

        let verifying_key = VerifyingKey::<Sha256>::new_unprefixed(self.0.to_public_key());
        let sig = Signature::try_from(signature).unwrap();
        verifying_key.verify(data, &sig).is_ok()
    }

    pub fn to_public_key(&self) -> Vec<u8> {
        self.0.to_public_key().n().to_bytes_be()
    }
}

impl Driver {
    pub fn generate_key() -> PrivateKey {
        let exponent = BigUint::from_bytes_be(&[0x01, 0x00, 0x01]);
        let mut rng = OsRng;
        let p_key =
            RsaPrivateKey::new_with_exp(&mut rng, 4096, &exponent).expect("Key generation failed");
        PrivateKey(p_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_key() {
        Driver::generate_key();
    }

    #[test]
    fn test_sign_verify() {
        let key = Driver::generate_key();
        let signature = key.sign(&[0; 16]);
        assert!(signature.len() > 0);
        let verified = key.verify(&[0; 16], &signature);
        assert!(verified);
    }
}
