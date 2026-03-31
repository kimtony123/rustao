use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Tag {
    pub name: String,
    pub value: String,
}

impl Tag {
    pub fn new(name: &str, value: &str) -> Self {
        Self {
            name: name.to_string(),
            value: value.to_string(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SendMessageOptions {
    pub encrypt_with_rsa: Option<Vec<u8>>, // RSA public key in PKCS#1 DER format
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseCu {
    pub output: serde_json::Value,
    pub messages: Vec<serde_json::Value>,
    pub spawns: Vec<serde_json::Value>,
    pub error: Option<String>,
    pub gas_used: u64,
}