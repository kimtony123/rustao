use crate::dataitem::DataItem;
use crate::encrypt::encrypt_payload;
use crate::error::{Error, Result};
use crate::schema::{ResponseCu, SendMessageOptions, Tag};
use crate::signer::ARSigner;
use crate::utils::base64url_encode;
use reqwest::Client as HttpClient;
use rsa::pkcs1::DecodeRsaPublicKey;
use serde_json::Value;
use std::time::Duration;

const DEFAULT_MU: &str = "https://mu.ao.arweave.net";
const DEFAULT_CU: &str = "https://cu.ao.arweave.net";
const DEFAULT_COMPUTE: &str = "https://push.forward.computer";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

pub struct Client {
    signer: ARSigner,
    pub mu: String,
    pub cu: String,
    pub compute_gateway: String,
    http_client: HttpClient,
}

impl Clone for Client {
    fn clone(&self) -> Self {
        Self {
            signer: self.signer.clone(),
            mu: self.mu.clone(),
            cu: self.cu.clone(),
            compute_gateway: self.compute_gateway.clone(),
            http_client: self.http_client.clone(),
        }
    }
}

impl Client {
    pub fn new(signer: ARSigner) -> Self {
        let http_client = HttpClient::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .expect("Failed to build HTTP client");
        Self {
            signer,
            mu: DEFAULT_MU.to_string(),
            cu: DEFAULT_CU.to_string(),
            compute_gateway: DEFAULT_COMPUTE.to_string(),
            http_client,
        }
    }

    pub fn with_mu(mut self, mu: &str) -> Self {
        self.mu = mu.to_string();
        self
    }

    pub fn with_cu(mut self, cu: &str) -> Self {
        self.cu = cu.to_string();
        self
    }

    pub fn with_compute_gateway(mut self, gateway: &str) -> Self {
        self.compute_gateway = gateway.to_string();
        self
    }

    pub async fn send_message(
        &self,
        process_id: &str,
        data: &[u8],
        tags: Vec<Tag>,
        anchor: Option<&[u8]>,
        options: Option<SendMessageOptions>,
    ) -> Result<String> {
        let mut data = data.to_vec();
        let mut tags = tags;

        if let Some(opts) = options {
            if let Some(rsa_pub_bytes) = opts.encrypt_with_rsa {
                let rsa_pub = rsa::RsaPublicKey::from_pkcs1_der(&rsa_pub_bytes)
                    .map_err(|e| Error::Encryption(e.to_string()))?;
                let (encrypted_data, encrypted_key, nonce) = encrypt_payload(&data, &rsa_pub)?;
                data = encrypted_data;
                tags.push(Tag::new("Encrypted-Key", &base64url_encode(&encrypted_key)));
                tags.push(Tag::new("Nonce", &base64url_encode(&nonce)));
                tags.push(Tag::new("Encryption-Algorithm", "AES-GCM+RSA-OAEP"));
            }
        }

        let anchor = anchor.unwrap_or(&[]);
        let mut di = DataItem::new(process_id.to_string(), data, tags, anchor.to_vec());
        di.sign(&self.signer)?;

        let url = format!("{}/submit", self.mu);
        let resp = self.http_client
            .post(&url)
            .header("Content-Type", "application/octet-stream")
            .timeout(DEFAULT_TIMEOUT)
            .body(di.serialize()?)
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            return Err(Error::HttpError(format!("MU returned {}", status)));
        }
        Ok(di.id)
    }

    pub async fn spawn_process(
        &self,
        module_tx_id: &str,
        data: &[u8],
        tags: Vec<Tag>,
        anchor: Option<&[u8]>,
        scheduler: Option<&str>,
    ) -> Result<String> {
        let mut spawn_tags = tags;
        spawn_tags.push(Tag::new("Type", "Process"));
        spawn_tags.push(Tag::new("Module", module_tx_id));
        let scheduler = scheduler.unwrap_or("_GQ33BkPtZrqxA84vM8Zk-N2aO0toNNu_C-l-rawrBA");
        spawn_tags.push(Tag::new("Scheduler", scheduler));

        let anchor = anchor.unwrap_or(&[]);
        let mut di = DataItem::new(String::new(), data.to_vec(), spawn_tags, anchor.to_vec());
        di.sign(&self.signer)?;

        let url = format!("{}/submit", self.mu);
        let resp = self.http_client
            .post(&url)
            .header("Content-Type", "application/octet-stream")
            .timeout(DEFAULT_TIMEOUT)
            .body(di.serialize()?)
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(Error::HttpError(format!("MU returned {}", resp.status())));
        }
        Ok(di.id)
    }

    pub async fn dry_run(
        &self,
        process_id: &str,
        data: &[u8],
        tags: Vec<Tag>,
        anchor: Option<&[u8]>,
        owner: Option<&str>,
    ) -> Result<ResponseCu> {
        let owner = owner.unwrap_or(self.signer.address());
        let anchor = anchor.map(|a| base64url_encode(a)).unwrap_or_default();
        let data_b64 = base64url_encode(data);
        let tags_json: Vec<Value> = tags.into_iter()
            .map(|t| serde_json::json!({"name": t.name, "value": t.value}))
            .collect();

        let payload = serde_json::json!({
            "Target": process_id,
            "Owner": owner,
            "Anchor": anchor,
            "Data": data_b64,
            "Tags": tags_json,
        });

        let url = format!("{}/dry-run?process-id={}", self.cu, process_id);
        let resp = self.http_client
            .post(&url)
            .json(&payload)
            .timeout(DEFAULT_TIMEOUT)
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(Error::HttpError(format!("CU returned {}", resp.status())));
        }
        let json: Value = resp.json().await?;
        Ok(ResponseCu {
            output: json.get("Output").cloned().unwrap_or(Value::Null),
            messages: json.get("Messages").cloned().unwrap_or(Value::Array(vec![])).as_array().unwrap().clone(),
            spawns: json.get("Spawns").cloned().unwrap_or(Value::Array(vec![])).as_array().unwrap().clone(),
            error: json.get("Error").and_then(|v| v.as_str()).map(String::from),
            gas_used: json.get("GasUsed").and_then(|v| v.as_u64()).unwrap_or(0),
        })
    }

    pub async fn get_compute(&self, process_id: &str, path: &str) -> Result<Vec<u8>> {
        let url = format!("{}/{}/compute/{}", self.compute_gateway, process_id, path);
        let resp = self.http_client
            .get(&url)
            .timeout(DEFAULT_TIMEOUT)
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(Error::HttpError(format!("Compute returned {}", resp.status())));
        }
        Ok(resp.bytes().await?.to_vec())
    }

    pub async fn get_compute_string(&self, process_id: &str, path: &str) -> Result<String> {
        let bytes = self.get_compute(process_id, path).await?;
        String::from_utf8(bytes).map_err(|e| Error::Other(e.to_string()))
    }

    pub async fn get_compute_json(&self, process_id: &str, path: &str) -> Result<Value> {
        let bytes = self.get_compute(process_id, path).await?;
        serde_json::from_slice(&bytes).map_err(Error::Json)
    }

    pub async fn wait_for_result(
        &self,
        message_id: &str,
        process_id: &str,
        timeout: Duration,
        poll_interval: Duration,
    ) -> Result<ResponseCu> {
        let deadline = tokio::time::Instant::now() + timeout;
        let url = format!("{}/result/{}?process-id={}", self.cu, message_id, process_id);

        while tokio::time::Instant::now() < deadline {
            let resp = self.http_client.get(&url).timeout(DEFAULT_TIMEOUT).send().await;
            match resp {
                Ok(r) if r.status().is_success() => {
                    let json: Value = r.json().await?;
                    return Ok(ResponseCu {
                        output: json.get("Output").cloned().unwrap_or(Value::Null),
                        messages: json.get("Messages").cloned().unwrap_or(Value::Array(vec![])).as_array().unwrap().clone(),
                        spawns: json.get("Spawns").cloned().unwrap_or(Value::Array(vec![])).as_array().unwrap().clone(),
                        error: json.get("Error").and_then(|v| v.as_str()).map(String::from),
                        gas_used: json.get("GasUsed").and_then(|v| v.as_u64()).unwrap_or(0),
                    });
                }
                Ok(r) if r.status() == 404 => {}
                Ok(r) => return Err(Error::HttpError(format!("CU returned {}", r.status()))),
                Err(e) => return Err(Error::Request(e)),
            }
            tokio::time::sleep(poll_interval).await;
        }
        Err(Error::Timeout)
    }
}