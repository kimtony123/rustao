use crate::error::{Error, Result};
use num_bigint_dig::BigUint;
use rsa::traits::PublicKeyParts;
use rsa::RsaPrivateKey; // Removed RsaPublicKey (unused)
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

pub struct ARSigner {
    private_key: RsaPrivateKey,
    public_key: Vec<u8>,
    address: String,
}

impl Clone for ARSigner {
    fn clone(&self) -> Self {
        Self {
            private_key: self.private_key.clone(),
            public_key: self.public_key.clone(),
            address: self.address.clone(),
        }
    }
}

impl ARSigner {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        Self::from_jwk(&content)
    }

    pub fn from_jwk(jwk: &str) -> Result<Self> {
        let v: Value = serde_json::from_str(jwk)?;

        let n = decode_jwk_field(&v, "n")?;
        let e = decode_jwk_field(&v, "e")?;
        let d = decode_jwk_field(&v, "d")?;
        let p = decode_jwk_field(&v, "p")?;
        let q = decode_jwk_field(&v, "q")?;

        let n = BigUint::from_bytes_be(&n);
        let e = BigUint::from_bytes_be(&e);
        let d = BigUint::from_bytes_be(&d);
        let p = BigUint::from_bytes_be(&p);
        let q = BigUint::from_bytes_be(&q);

        let private_key = RsaPrivateKey::from_components(n, e, d, vec![p, q])
            .map_err(|e| Error::InvalidWallet(e.to_string()))?;
        let public_key = private_key.to_public_key();

        let pub_modulus = public_key.n().to_bytes_be();
        let hash = Sha256::digest(&pub_modulus);
        let address = base58::ToBase58::to_base58(&hash[..]);

        Ok(Self {
            private_key,
            public_key: pub_modulus,
            address,
        })
    }

    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>> {
        use rsa::pkcs1v15::{Signature, SigningKey};
        use rsa::signature::{RandomizedSigner, SignatureEncoding};

        let mut rng = rand::thread_rng();
        let signing_key = SigningKey::<Sha256>::new_unprefixed(self.private_key.clone());
        let signature: Signature = signing_key.sign_with_rng(&mut rng, data);
        Ok(signature.to_bytes().to_vec())
    }

    pub fn public_key(&self) -> &[u8] {
        &self.public_key
    }

    pub fn address(&self) -> &str {
        &self.address
    }
}

fn decode_jwk_field(v: &Value, field: &str) -> Result<Vec<u8>> {
    let s = v[field]
        .as_str()
        .ok_or_else(|| Error::InvalidWallet(format!("missing {}", field)))?;
    crate::utils::base64url_decode(s).map_err(|e| Error::InvalidWallet(e.to_string()))
}
